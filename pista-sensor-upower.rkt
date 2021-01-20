#! /usr/bin/env racket
#lang racket

(struct device
        (path))

(struct line-power
        (path online))

(struct battery
        (path state energy energy-full percentage))

; parsing = #f
;         | device?
;         | battery?
;         | line-power?
(struct state
        (parsing plugged-in? batteries))

(define (display-device? line)
  (regexp-match? #rx"/DisplayDevice$" line))

(define unique (compose set->list list->set))

(define/contract (aggregate plugged-in? batteries)
  (-> boolean? (listof battery?) string?)
  (let ([direction
          (let ([states (map battery-state batteries)])
               (cond [(not plugged-in?)                           "<"]
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

(define (loop input s0)
  (define line (read-line input))
  (unless (eof-object? line)
          (let* ([fields (string-split line)]
                 [p0 (state-parsing s0)]
                 [s1
                   (cond
                     ; BOM when --dump
                     [(regexp-match? #rx"^Device:[ \t]+" line)
                      (struct-copy state s0 [parsing (device (second fields))])]

                     ; BOM when --monitor-detail
                     [(regexp-match?
                        #rx"^\\[[0-9]+:[0-9]+:[0-9]+\\.[0-9]+\\][ \t]+device changed:[ \t]+"
                        line)
                      (struct-copy state s0 [parsing (device (fourth fields))])]

                     ; EOM
                     [(regexp-match? #rx"^$" line)
                      ; Use DisplayDevice if we have it, but do not stash it,
                      ; because:
                      ; 1. it is already an aggregate;
                      ; 2. it is not reported by "--monitor-detail" - we expect
                      ;    to see it only once, because of the initial "--dump".
                      (let* ([s1
                               (cond [(and (battery? p0)
                                           ; Ignoring DisplayDevice:
                                           (not (display-device? (battery-path p0))))
                                      (struct-copy state s0
                                                   [batteries
                                                     (dict-set (state-batteries s0)
                                                               (battery-path p0)
                                                               p0)])]
                                     [(line-power? p0)
                                      (struct-copy state s0
                                                   [plugged-in?
                                                     (match (line-power-online p0)
                                                            ["yes" #t]
                                                            ["no"  #f])])]
                                     [else s0])]
                             [batteries
                               ; Using DisplayDevice aggregate instead of computing our own:
                               (if (and (battery? p0)
                                        (display-device? (battery-path p0)))
                                   (list p0)
                                   (dict-values (state-batteries s1)))])
                            (with-handlers
                              ; Expect broken pipes
                              ([exn:fail:filesystem:errno?
                                 (λ (e) (eprintf "[error] Print failed: ~v\n" e))])
                              (displayln (aggregate (state-plugged-in? s1) batteries))
                              (flush-output))
                            (struct-copy state s1 [parsing #f]))]

                     ; -- BEGIN battery
                     [(and (device? p0)
                           (regexp-match? #rx"^  battery$" line))
                      (define p1 (battery (device-path p0) #f #f #f #f))
                      (struct-copy state s0 [parsing p1])]

                     [(and (battery? p0)
                           (regexp-match? #rx"^    state:" line))
                      (define p1 (struct-copy battery p0 [state (second fields)]))
                      (struct-copy state s0 [parsing p1])]

                     [(and (battery? p0)
                           (regexp-match? #rx"^    energy:" line))
                      (define e (string->number (second fields)))
                      (define p1 (struct-copy battery p0 [energy e]))
                      (struct-copy state s0 [parsing p1])]

                     [(and (battery? p0)
                           (regexp-match? #rx"^    energy-full:" line))
                      (define ef (string->number (second fields)))
                      (define p1 (struct-copy battery p0 [energy-full ef]))
                      (struct-copy state s0 [parsing p1])]

                     [(and (battery? p0)
                           (regexp-match? #rx"^    percentage:" line))
                      (define pct (second fields))
                      (define p1 (struct-copy battery p0 [percentage pct]))
                      (struct-copy state s0 [parsing p1])]
                     ; -- END battery

                     ; -- BEGIN line-power
                     [(and (device? p0) (regexp-match? #rx"^  line-power$" line))
                      (define dp (device-path p0))
                      (struct-copy state s0 [parsing (line-power dp #f)])]

                     [(and (line-power? p0) (regexp-match? #rx"^    online:" line))
                      (define o (second fields))
                      (define p1 (struct-copy line-power p0 [online o]))
                      (struct-copy state s0 [parsing p1])]
                     ; -- END line-power

                     [else s0])])
                (loop input s1))))

(define (start)
  (define cmd "stdbuf -o L upower --dump; stdbuf -o L upower --monitor-detail")
  (match-define (list in-port out-port pid in-err-port ctrl) (process cmd))
  (loop in-port (state #f #f '()))
  (define code (ctrl 'exit-code))
  (define stderr (port->string in-err-port))
  (when (> (string-length stderr) 0)
    (eprintf "[error] upower command stderr: ~v~n" stderr))
  (exit code))

(module+ main
  (start))
