#! /usr/bin/env racket

#lang racket

; Can we do better than "types"? I hate that I've become That Guy ...
(module types racket
  (provide (all-defined-out))

  (struct device
          (path native-path))

  (struct line-power
          (path online)
          #:transparent)

  (struct battery
          (path state energy energy-full)
          #:transparent)

  (struct status
          (direction percentage)
          #:transparent)
  ) ; Does this make you angry?

(module state racket
  (provide state-init
           state-update-plugged-in
           state-update-batteries
           state->status)

  (require (submod ".." types))

  (struct state
          (plugged-in? batteries clock) ; clock is just for debugging
          #:transparent)

  (define (state-init)
    (state #f '() 0))

  (define (clock-incr s)
    (struct-copy state s [clock (+ 1 (state-clock s))]))

  (define/contract (state-update-batteries s b)
    (-> state? battery? state?)
    (define batteries (dict-set (state-batteries s) (battery-path b) b))
    (clock-incr (struct-copy state s [batteries batteries])))

  (define/contract (state-update-plugged-in s online)
    (-> state? (or/c "yes" "no") state?)
    (define plugged-in? (match online ["yes" #t] ["no" #f]))
    (clock-incr (struct-copy state s [plugged-in? plugged-in?])))

  (define unique (compose set->list list->set))

  (define/contract (state->status s)
    (-> state? status?)
    (define batteries (dict-values (state-batteries s)))
    (let ([direction
            (let ([states (map battery-state batteries)])
              (cond [(not (state-plugged-in? s))                 '<]
                    [(member      "discharging"         states)  '<]
                    [(member         "charging"         states)  '>]
                    [(equal? '("fully-charged") (unique states)) '=]
                    [else                                        '?]))]
          [percentage
            (if (empty? batteries)
                #f
                (let ([cur (apply + (map battery-energy batteries))]
                      [max (apply + (map battery-energy-full batteries))])
                  (* 100 (/ cur max))))])
      (status direction percentage)))
  )

(require 'types
         'state)

(define/contract (status->string s)
  (-> status? string?)
  (match-define (status direction percentage) s)
  (format "(⚡ ~a~a%)" direction (if percentage
                                    (~r percentage #:precision 0 #:min-width 3)
                                    "___")))

(define/contract (safe-print s)
  (-> string? void?)
  ; We expect occasional broken pipes:
  (with-handlers
    ([exn? (λ (e) (log-error "Print failure. Exception: ~v" e))])
    (displayln s)
    (flush-output)))

(define/contract (read-msg input)
  (-> input-port? (or/c 'eof battery? line-power?))
  ; msg = #f
  ;     | device?
  ;     | battery?
  ;     | line-power?
  (define (next msg)
    (define line (read-line input))
    (if (eof-object? line)
        'eof
        ; TODO can we make fields lazy? To avoid splitting unmatched lines.
        (let ([fields (string-split line)])
          (cond
            ; EOM
            [(string=? line "")
             (if msg
                 (begin
                   (log-debug "msg: ~v" msg)
                   msg)
                 (begin
                   (log-debug "EOM for unknown msg")
                   (next msg)))]

            ; BOM when --dump
            [(and (not msg)
                  (string-prefix? line "Device: "))
             (next (device (second fields) #f))]

            ; BOM when --monitor-detail
            [(and (not msg)
                  (regexp-match?
                    #rx"^\\[[0-9]+:[0-9]+:[0-9]+\\.[0-9]+\\][ \t]+device changed:[ \t]+"
                    line))
             (next (device (fourth fields) #f))]

            [(and (device? msg)
                  (string-prefix? line "  native-path:"))
             (next (struct-copy device msg [native-path (second fields)]))]

            ; -- BEGIN battery
            [(and (device? msg)
                  (string=? line "  battery"))
             (let ([path (device-path msg)]
                   [native-path (device-native-path msg)])
               (next (battery (if native-path native-path path) #f #f #f)))]

            [(and (battery? msg)
                  (string-prefix? line "    state:"))
             (next (struct-copy battery msg [state (second fields)]))]

            [(and (battery? msg)
                  (string-prefix? line "    energy:"))
             (next (struct-copy battery msg [energy
                                              (string->number (second fields))]))]

            [(and (battery? msg)
                  (string-prefix? line "    energy-full:"))
             (next (struct-copy battery msg [energy-full
                                              (string->number (second fields))]))]
            ; -- END battery

            ; -- BEGIN line-power
            [(and (device? msg) (string=? line "  line-power"))
             (let ([path (device-path msg)]
                   [native-path (device-native-path msg)])
               (next (line-power (if native-path native-path path) #f)))]

            [(and (line-power? msg) (string-prefix? line "    online:"))
             (next (struct-copy line-power msg [online (second fields)]))]
            ; -- END line-power

            [else
              (next msg)]))))
  (next #f))

(define (start-parser input printer)
  (log-info "Starting loop ...")
  (let loop ([s (state-init)])
    (log-debug "parser state: ~v" s)
    (thread-send printer (state->status s))
    (match (read-msg input)
      ['eof
       (thread-send printer 'parser-exit)]
      [(struct* battery ([path p])) #:when (string-suffix? p "/DisplayDevice")
       (loop s)]
      [(and b (struct* battery ()))
       (loop (state-update-batteries s b))]
      [(line-power _ online)
       (loop (state-update-plugged-in s online))])))

(define (timer-start seconds msg)
  (let ([parent (current-thread)])
    (thread (λ () (sleep seconds) (thread-send parent msg)))))

(define timer-cancel
  kill-thread)

(define (start-printer interval)
  (local-require libnotify)
  ; TODO User-defined alerts
  (define init-discharging-alerts (sort '(100 70 50 30 20 15 10 5 4 3 2 1 0) <))
  (log-info "Alerts defined: ~v" init-discharging-alerts)
  (let loop ([last-status #f]
             [alerts      init-discharging-alerts])
    (let ([tm (timer-start interval 'print)])
      (match (thread-receive)
        [(and new-status (status direction percentage))
         (timer-cancel tm)
         (log-debug "New status: ~v" new-status)
         ; TODO Fully-charged alert
         (let ([alerts
                 (cond [(and percentage (equal? '< direction))
                        (match (dropf alerts (λ (a) (<= a percentage)))
                          [(cons a _)
                           (send (new notification%
                                      ; TODO User-defined summary
                                      [summary (format "Battery power bellow ~a%!" a)]

                                      ; TODO User-defined body
                                      [body (~r percentage #:precision 2)]

                                      ; TODO User-defined urgency
                                      [urgency (cond [(<= a 10) 'critical]
                                                     [(<= a 30) 'normal]
                                                     [else      'low])])
                                 show)
                           (log-info "Alert sent: ~v" a)
                           (filter (λ (a-i) (< a-i a)) alerts)]
                          [_
                            alerts])]
                       [else
                         init-discharging-alerts])])
           (log-info "Alerts remaining: ~v" alerts)
           (loop (status->string new-status) alerts))]
        ['print #:when last-status
         (safe-print last-status)
         (loop last-status alerts)]
        ['print
         (log-warning "Time to print, before ever receiving a status!")
         (loop last-status alerts)]
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

(define (run log-level interval)
  (start-logger log-level)
  ; TODO Multiplex ports so we can execute as separate executables instead
  (define cmd "stdbuf -o L upower --dump; stdbuf -o L upower --monitor-detail")
  (log-info "Spawning command: ~v" cmd)
  (match-define (list in-port out-port pid in-err-port ctrl) (process cmd))
  (log-info "Child process PID: ~a" pid)
  (let* ([printer    (thread (λ () (start-printer interval)))]
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
  (define opt-interval 1)
  (command-line #:once-each
                [("-d" "--debug")
                 "Enable debug logging"
                 (set! opt-log-level 'debug)]
                [("-i" "--interval")
                 i "Maximum interval between state prints"
                 (set! opt-interval (string->number i))])
  (run opt-log-level opt-interval))
