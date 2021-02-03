; MPD API reference:
; https://www.musicpd.org/doc/html/protocol.html
;
#lang typed/racket

(require "sensor.rkt")

(define-type State
  (U 'play
     'pause
     'stop))

(define-type Cmd
  'status)

(define-type Msg
  (Immutable-HashTable String String))

(struct status
        ([state           : State]
         [seconds-current : Nonnegative-Integer]
         [seconds-total   : Nonnegative-Integer])
        #:type-name Status)

(: read-msg (-> Input-Port Msg))
(define (read-msg ip)
  (let loop ([msg : Msg #hash()])
    (define line (read-line ip))
    (log-debug "Msg line read: ~v" line)
    (cond [(eof-object? line) (assert #f)]
          [(string-prefix? line "OK") msg]
          [else
            (match (regexp-match #rx"^([A-Za-z-]+)(: +)(.*$)" line)
              [(list _ k _ v) #:when (and (string? k)
                                          (string? v))
               (loop (hash-set msg k v))])])))

(: send/recv (-> Input-Port Output-Port Cmd Msg))
(define (send/recv ip op cmd)
  (displayln cmd op)
  (flush-output op)
  (define msg (read-msg ip))
  msg)

(: msg->status (-> Msg Status))
(define (msg->status msg)
  (log-debug "(msg->status ~a)" (pretty-format msg))
  (define time (map string->number (string-split (hash-ref msg "time") ":")))
  (define state (match (hash-ref msg "state")
                  ["play"  'play]
                  ["pause" 'pause]
                  ["stop"  'stop]))
  (status state
          (cast (first  time) Nonnegative-Integer)
          (cast (second time) Nonnegative-Integer)))

(: state->symbol (-> State Symbol))
(define (state->symbol s)
  (match s
    ['play  '>]
    ['pause '=]
    ['stop  '-]))

(: status->string (-> Status String))
(define (status->string s)
  (define time
    (let* ([s   (status-seconds-current s)]
           [h   (floor (/ (/ s 60) 60))]
           [s   (- s (* 60 (* 60 h)))] ; seconds beyond hours
           [m   (floor (/ s 60))]
           [s   (- s (* 60 m))]  ; seconds beyond minutes
           [fmt (λ ([t : Real]) (~r t #:precision 0 #:min-width 2 #:pad-string "0"))]
           [hh  (if (> h 0) `(,(fmt h)) '())]
           [mm  `(,(fmt m))]
           [ss  `(,(fmt s))])
      (string-join (append hh mm ss) ":")))
  (define percentage
    (let ([cur (status-seconds-current s)]
          [tot (status-seconds-total s)])
      (if (> (status-seconds-total s) 0)
          (format "~a%" (~r (* 100 (/ cur tot)) #:precision 0 #:min-width 3))
          "~")))
  (format "(~a ~a ~a)"
          (state->symbol (status-state s))
          (~a time #:width 8 #:align 'right)
          percentage))

(: main (->* (#:host String #:port Integer Nonnegative-Real) () Void))
(define (main #:host host #:port port interval)
  (with-handlers
    ([exn:fail:network?
       (λ (_)
          ; TODO Connection retry loop
          (log-fatal "Connection could not be established to: ~a ~a" host port))])
    (define-values (ip op) (tcp-connect host port))
    (let ([init-line (read-line ip)])
      (log-info "Server version: ~v" init-line))
    (let loop ()
      (displayln (status->string (msg->status (send/recv ip op 'status))))
      (flush-output)
      (sleep interval)
      (loop))
    (close-input-port ip)
    (close-output-port op))
  (flush-output (current-error-port)))

(module+ main
  (define opt-host "localhost")
  (define opt-port 6600)
  (define opt-log-level : Log-Level 'info)
  (define opt-interval-seconds : Nonnegative-Real 1)
  (command-line
    #:once-each
    [("-d" "--debug")
     "Enable debug logging"
     (set! opt-log-level 'debug)]
    [("-i" "--interval")
     i "Poll interval"
     (set! opt-interval-seconds
           (cast (string->number (cast i String)) Nonnegative-Real))])
  (define log-handler (logger-start opt-log-level))
  (main
    #:host opt-host
    #:port opt-port
    opt-interval-seconds))
