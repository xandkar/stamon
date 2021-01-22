#! /usr/bin/env racket

#lang racket

(struct device
        (path native-path))

(struct line-power
        (path online)
        #:transparent)

(struct battery
        (path state energy energy-full)
        #:transparent)

(struct state
        (plugged-in? batteries)
        #:transparent)

(struct status
        (direction percentage)
        #:transparent)

(define (display-device? line)
  (regexp-match? #rx"/DisplayDevice$" line))

(define unique (compose set->list list->set))

(define/contract (status->string s)
  (-> status? string?)
  (match-define (status direction percentage) s)
  (format "(⚡ ~a~a%)" direction (if percentage
                                    (~r percentage #:precision 0 #:min-width 3)
                                    "___")))

(define/contract (state->status s)
  (-> state? status?)
  (define batteries (dict-values (state-batteries s)))
  (let ([direction
          (let ([states (map battery-state batteries)])
            (cond [(not (state-plugged-in? s))                 "<"]
                  [(member      "discharging"         states)  "<"]
                  [(member         "charging"         states)  ">"]
                  [(equal? '("fully-charged") (unique states)) "="]
                  [else                                        "?"]))]
        [percentage
          (if (empty? batteries)
              #f
              (let ([cur (apply + (map battery-energy batteries))]
                    [max (apply + (map battery-energy-full batteries))])
                (* 100 (/ cur max))))])
    (status direction percentage)))

(define state->string (compose status->string state->status))

(define/contract (safe-print s)
  (-> string? void?)
  (with-handlers
    ; Expect broken pipes
    ([exn:fail:filesystem:errno?
       (λ (e) (log-error "print failed: ~v\n" e))])
    (displayln s)
    (flush-output)))

(define/contract (read-msg input)
  (-> input-port? (or/c #f battery? line-power?))
  ; msg = #f
  ;     | device?
  ;     | battery?
  ;     | line-power?
  (define (next msg)
    (define line (read-line input))
    (if (eof-object? line)
        #f
        ; TODO can we make fields lazy? To avoid splitting unmatched lines.
        (let ([fields (string-split line)])
          (cond
            ; EOM
            [(regexp-match? #rx"^$" line)
             (if msg
                 (begin
                   (log-debug "msg: ~v" msg)
                   msg)
                 (begin
                   (log-debug "EOM for unknown msg")
                   (next msg)))]

            ; BOM when --dump
            [(and (not msg)
                  (regexp-match? #rx"^Device:[ \t]+" line))
             (next (device (second fields) #f))]

            ; BOM when --monitor-detail
            [(and (not msg)
                  (regexp-match?
                    #rx"^\\[[0-9]+:[0-9]+:[0-9]+\\.[0-9]+\\][ \t]+device changed:[ \t]+"
                    line))
             (next (device (fourth fields) #f))]

            [(and (device? msg)
                  (regexp-match? #rx"^  native-path:" line))
             (next (struct-copy device msg [native-path (second fields)]))]

            ; -- BEGIN battery
            [(and (device? msg)
                  (regexp-match? #rx"^  battery$" line))
             (let ([path (device-path msg)]
                   [native-path (device-native-path msg)])
               (next (battery (if native-path native-path path) #f #f #f)))]

            [(and (battery? msg)
                  (regexp-match? #rx"^    state:" line))
             (next (struct-copy battery msg [state (second fields)]))]

            [(and (battery? msg)
                  (regexp-match? #rx"^    energy:" line))
             (next (struct-copy battery msg [energy
                                              (string->number (second fields))]))]

            [(and (battery? msg)
                  (regexp-match? #rx"^    energy-full:" line))
             (next (struct-copy battery msg [energy-full
                                              (string->number (second fields))]))]
            ; -- END battery

            ; -- BEGIN line-power
            [(and (device? msg) (regexp-match? #rx"^  line-power$" line))
             (let ([path (device-path msg)]
                   [native-path (device-native-path msg)])
               (next (line-power (if native-path native-path path) #f)))]

            [(and (line-power? msg) (regexp-match? #rx"^    online:" line))
             (next (struct-copy line-power msg [online (second fields)]))]
            ; -- END line-power

            [else
              (next msg)]))))
  (next #f))

(define (start-parser input printer)
  (log-info "Starting loop ...")
  (let loop ([s (state #f '())])
    (log-debug "parser state: ~v" s)
    (thread-send printer (state->status s))
    (match (read-msg input)
      [#f
        (thread-send printer 'parser-exit)]
      [(struct* battery ([path p])) #:when (display-device? p)
       (loop s)]
      [(and b (struct* battery ([path p])))
       (loop (struct-copy state s
                          [batteries (dict-set (state-batteries s) p b)]))]
      [(line-power _ online)
       (loop (struct-copy state s [plugged-in?
                                    (match online
                                      ["yes" #t]
                                      ["no" #f])]))])))

(define (timer-start seconds msg)
  (let ([parent (current-thread)])
    (thread (λ () (sleep seconds) (thread-send parent msg)))))

(define timer-cancel
  kill-thread)

(define (start-printer max-interval)
  (let loop ([prev #f])
    (let ([tm (timer-start max-interval 'timeout)])
      (match (thread-receive)
        [(and curr (struct* status ()))
         (timer-cancel tm)
         (log-debug "printer status: ~v" curr)
         (let ([curr (status->string curr)])
           (safe-print curr)
           (loop curr))]
        ['timeout #:when prev
         (log-info "Timeout. Reprinting previous status: ~v" prev)
         (safe-print prev)
         (loop prev)]
        ['timeout
         (log-warning "Timeout before ever receiving a status!" prev)
         (loop prev)]
        ['parser-exit
         (void)]))))

(define (start-logger level)
  ; TODO implement graceful stop, flushing before exiting
  (define logger (make-logger #f #f level #f))
  (define log-receiver (make-log-receiver logger level))
  (thread
    (λ ()
       (local-require racket/date)
       (date-display-format 'iso-8601)
       (let loop ()
         (match-let ([(vector level msg _ ...) (sync log-receiver)])
           (eprintf "~a [~a] ~a~n" (date->string (current-date) #t) level msg))
         (loop))))
  (current-logger logger))

(define (run log-level max-interval)
  (start-logger log-level)
  (define cmd "stdbuf -o L upower --dump; stdbuf -o L upower --monitor-detail")
  (log-info "Spawning command: ~v" cmd)
  (match-define (list in-port out-port pid in-err-port ctrl) (process cmd))
  (log-info "Child process PID: ~a" pid)
  (let* ([printer    (thread (λ () (start-printer max-interval)))]
         [parser     (thread (λ () (start-parser in-port printer)))]
         [cmd-logger (thread (λ () (let loop ()
                                     (let ([line (read-line in-err-port)])
                                       (unless (eof-object? line)
                                         (log-error "upower stderr: ~v~n" line)
                                         (loop))))))])
    (for-each thread-wait (list parser
                                printer
                                cmd-logger)))
  (ctrl 'wait)
  (define code (ctrl 'exit-code))
  (log-info "upower exit code: ~a" code)
  (when (> code 0)
    ; FIXME We exit faster than the logger can print. Need to flush before exit.
    (log-error "non-zero exit code from upower: ~a" code))
  (exit code))

(module+ main
  (define opt-log-level 'info)
  (define opt-interval 30)
  (command-line #:once-each
                [("-d" "--debug")
                 "Enable debug logging"
                 (set! opt-log-level 'debug)]
                [("-i" "--interval")
                 i "Maximum interval between state prints"
                 (set! opt-interval (string->number i))])
  (run opt-log-level opt-interval))
