#! /usr/bin/env racket

#lang typed/racket

; Can we do better than "types"? I hate that I've become That Guy ...
(module types typed/racket
  (provide (all-defined-out))

  (struct device
          ([path        : String]
           [native-path : (Option String)]))

  (struct line-power
          ([path   : String]
           [online : Boolean])
          #:transparent)

  (struct battery
          ([path        : String]
           [state       : (Option String)]
           [energy      : (Option Real)]
           [energy-full : (Option Real)])
          #:transparent)

  (struct status
          ([direction  : (U '= '< '> '?)]
           [percentage : (Option Real)])
          #:transparent)

  (define-type Batteries
    (Immutable-HashTable String battery))
  ) ; Does this make you angry?

(module state typed/racket
  (provide state-init
           state-update-plugged-in
           state-update-batteries
           state->status)

  (require (submod ".." types))

  (struct state
          ([plugged-in? : Boolean]
           [batteries   : Batteries]
           [clock       : Natural]) ; clock is just for debugging
          #:transparent)

  (: state-init (-> state))
  (define (state-init)
    (state #f #hash() 0))

  (: clock-incr (-> state state))
  (define (clock-incr s)
    (struct-copy state s [clock (+ 1 (state-clock s))]))

  (: state-update-batteries (-> state battery state))
  (define (state-update-batteries s b)
    (define batteries (hash-set (state-batteries s) (battery-path b) b))
    (clock-incr (struct-copy state s [batteries batteries])))

  (: state-update-plugged-in (-> state Boolean state))
  (define (state-update-plugged-in s online)
    (clock-incr (struct-copy state s [plugged-in? online])))

  (: unique (∀ (α) (-> (Listof α) (Listof α))))
  (define (unique xs)
    (set->list (list->set xs)))

  (: state->status (-> state status))
  (define (state->status s)
    (define batteries (hash-values (state-batteries s)))
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
                (let ([cur (apply + (filter-map battery-energy batteries))]
                      [max (apply + (filter-map battery-energy-full batteries))])
                  (* 100 (/ cur max))))])
      (status direction percentage)))
  )

(module notify racket
  (provide notify)

  (require libnotify)

  (define/contract (notify summary body urgency)
    (-> string? string? (or/c 'critical 'normal 'low) void?)
    (send (new notification%
               [summary summary]
               [body    body]
               [urgency urgency])
          show)))

(require 'types
         'state)

(require/typed 'notify
               [notify (-> String String (U 'critical 'normal 'low) Void)])

(require/typed "sensor.rkt"
               [sensor:logger-start (-> Log-Level Void)]
               [sensor:print/retry  (->* (String) (Natural) Void)])

(: status->string (-> status String))
(define (status->string s)
  (match-define (status direction percentage) s)
  (format "(⚡ ~a~a%)" direction (if percentage
                                    (~r percentage #:precision 0 #:min-width 3)
                                    "___")))

(: read-msg (-> Input-Port (U 'eof battery line-power)))
(define (read-msg input)
  ; msg = #f
  ;     | device?
  ;     | battery?
  ;     | line-power?
  (: next (-> (Option (U device line-power battery))
              (U 'eof line-power battery)))
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
                   (cast msg (U battery line-power)))
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
             (next (struct-copy battery
                                msg
                                [energy
                                  (cast (string->number (second fields)) Real)]))]

            [(and (battery? msg)
                  (string-prefix? line "    energy-full:"))
             (next (struct-copy battery
                                msg
                                [energy-full
                                  (cast (string->number (second fields)) Real)]))]
            ; -- END battery

            ; -- BEGIN line-power
            [(and (device? msg) (string=? line "  line-power"))
             (let ([path (device-path msg)]
                   [native-path (device-native-path msg)])
               (next (line-power (if native-path native-path path) #f)))]

            [(and (line-power? msg) (string-prefix? line "    online:"))
             (next (struct-copy line-power msg [online (match (second fields)
                                                         ["yes" #t]
                                                         ["no" #f])]))]
            ; -- END line-power

            [else
              (next msg)]))))
  (next #f))

(: start-parser (-> Input-Port Thread Void))
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
      ; TODO (: state-update (-> State Msg State))
      [(and b (struct* battery ()))
       (loop (state-update-batteries s b))]
      [(line-power _ online)
       (loop (state-update-plugged-in s online))])))

(: start-printer (-> Void))
(define (start-printer)
  ; TODO User-defined alerts
  (define init-discharging-alerts (sort '(100 70 50 30 20 15 10 5 4 3 2 1 0) <))
  (log-info "Alerts defined: ~v" init-discharging-alerts)
  (let loop ([printer : (Option Thread)  #f]
             [alerts                     init-discharging-alerts])
    (for-each (λ (a) (assert a natural?)) alerts)
    (match (thread-receive)
      [(and s (status direction percentage))
       (log-debug "New status: ~v" s)
       (when printer
         (kill-thread printer))
       ; TODO Fully-charged alert
       (let ([printer
               (thread (λ () (sensor:print/retry (status->string s))))]
             [alerts
               (cond [(and percentage (equal? '< direction))
                      (match (dropf alerts (λ ([a : Real]) (<= a percentage)))
                        [(cons a _)
                         (notify
                           ; TODO User-defined summary
                           (format "Battery power bellow ~a%!" a)
                           ; TODO User-defined body
                           (~r percentage #:precision 2)
                           ; TODO User-defined urgency
                           (cond [(<= a 10) 'critical]
                                 [(<= a 30) 'normal]
                                 [else      'low]))
                         (let ([alerts (filter (λ ([a-i : Real]) (< a-i a)) alerts)])
                           (log-info "Alert sent: ~a. Remaining: ~v" a alerts)
                           alerts)]
                        [_
                          alerts])]
                     [else
                       init-discharging-alerts])])
         (loop printer alerts))]
      ['parser-exit
       (void)])))

(: run (-> Log-Level Void))
(define (run log-level)
  (sensor:logger-start log-level)
  ; TODO Multiplex ports so we can execute as separate executables instead
  (define cmd "stdbuf -o L upower --dump; stdbuf -o L upower --monitor-detail")
  (log-info "Spawning command: ~v" cmd)
  (match-define (list in-port out-port pid in-err-port ctrl) (process cmd))
  (log-info "Child process PID: ~a" pid)
  (let* ([printer    (thread (λ () (start-printer)))]
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
  (when (and code (> code 0))
    ; FIXME We exit faster than the logger can print. Need to flush before exit.
    (log-error "non-zero exit code from upower: ~a" code))
  (exit code))

(module+ main
  (define opt-log-level : Log-Level 'info)
  (command-line #:once-each
                [("-d" "--debug")
                 "Enable debug logging"
                 (set! opt-log-level 'debug)])
  (run opt-log-level))
