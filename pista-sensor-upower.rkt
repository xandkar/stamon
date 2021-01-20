#! /usr/bin/env racket

#lang racket

(struct device
        (path))

(struct line-power
        (path online)
        #:transparent)

(struct battery
        (path state energy energy-full)
        #:transparent)

(struct state
        (plugged-in? batteries)
        #:transparent)

(define (display-device? line)
  (regexp-match? #rx"/DisplayDevice$" line))

(define unique (compose set->list list->set))

(define/contract (state->string s)
  (-> state? string?)
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
              "___"
              (let ([cur (apply + (map battery-energy batteries))]
                    [max (apply + (map battery-energy-full batteries))])
                (~r (* 100 (/ cur max))
                    #:precision 0
                    #:min-width 3)))])
    (format "(⚡ ~a~a%)" direction percentage)))

(define/contract (state-print s)
  (-> state? void?)
  (with-handlers
    ; Expect broken pipes
    ([exn:fail:filesystem:errno?
       (λ (e) (log-error "print failed: ~v\n" e))])
    (displayln (state->string s))
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
                 (next msg))]

            ; BOM when --dump
            [(and (not msg)
                  (regexp-match? #rx"^Device:[ \t]+" line))
             (next (device (second fields)))]

            ; BOM when --monitor-detail
            [(and (not msg)
                  (regexp-match?
                    #rx"^\\[[0-9]+:[0-9]+:[0-9]+\\.[0-9]+\\][ \t]+device changed:[ \t]+"
                    line))
             (next (device (fourth fields)))]

            ; -- BEGIN battery
            [(and (device? msg)
                  (regexp-match? #rx"^  battery$" line))
             (next (battery (device-path msg) #f #f #f))]

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
             (next (line-power (device-path msg) #f))]

            [(and (line-power? msg) (regexp-match? #rx"^    online:" line))
             (next (struct-copy line-power msg [online (second fields)]))]
            ; -- END line-power

            [else
              (next msg)]))))
  (next #f))

(define (run input)
  (let loop ([s (state #f '())])
    (log-debug "state: ~v" s)
    (state-print s)
    (match (read-msg input)
      [#f (void)]
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

(define (start-logger level)
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

(define (start)
  (start-logger 'debug)
  (define cmd "stdbuf -o L upower --dump; stdbuf -o L upower --monitor-detail")
  (match-define (list in-port out-port pid in-err-port ctrl) (process cmd))
  (run in-port)
  (define code (ctrl 'exit-code))
  (define stderr (port->string in-err-port))
  (when (> (string-length stderr) 0)
    (log-error "upower stderr: ~v~n" stderr))
  (exit code))

(module+ main
  (start))
