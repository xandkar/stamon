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

(struct conn
        ([ip : Input-Port]
         [op : Output-Port])
        #:type-name Conn)

(struct status
        ([state    : State]
         [elapsed  : Nonnegative-Real]
         [duration : Nonnegative-Real])
        #:type-name Status)

(: conn-open (-> String Integer Conn))
(define (conn-open host port)
  (define-values (ip op) (tcp-connect host port))
  (let ([server-version-line (read-line ip)])
    (log-info "Connected to: ~v" server-version-line))
  (conn ip op))

(: conn-close (-> Conn Void))
(define (conn-close c)
  (close-input-port  (conn-ip c))
  (close-output-port (conn-op c)))

(: recv (-> Input-Port Msg))
(define (recv ip)
  (let loop ([msg : Msg #hash()])
    (define line (read-line ip))
    (log-debug "Msg line read: ~v" line)
    (cond [(eof-object? line)
           (error 'eof-on-recv)]
          [(string-prefix? line "OK")
           msg]
          [else
            (match (regexp-match #rx"^([A-Za-z-]+)(: +)(.*$)" line)
              [(list _ k _ v) #:when (and (string? k)
                                          (string? v))
               (loop (hash-set msg k v))])])))

(: send (-> Output-Port Cmd Void))
(define (send op cmd)
  (displayln cmd op)
  (flush-output op))

(: send/recv (-> Conn Cmd Msg))
(define (send/recv c cmd)
  (send (conn-op c) cmd)
  (recv (conn-ip c)))

(: msg->status (-> Msg Status))
(define (msg->status msg)
  (log-debug "(msg->status ~a)" (pretty-format msg))
  (define state (match (hash-ref msg "state")
                  ["play"  'play]
                  ["pause" 'pause]
                  ["stop"  'stop]))
  ; TODO Restructure to handle #f rather than hand-waving a 0
  (define elapsed  (string->number (hash-ref msg "elapsed"  (λ () "0"))))
  (define duration (string->number (hash-ref msg "duration" (λ () "0"))))
  (status state
          (cast elapsed  Nonnegative-Real)
          (cast duration Nonnegative-Real)))

(: state->string (-> State String))
(define (state->string s)
  (match s
    ['play  ">"]
    ['pause "="]
    ['stop  "-"]))

(define seconds-in-minute : Natural 60)
(define seconds-in-hour   : Natural (* 60 seconds-in-minute))

(: status->string (-> Status String))
(define (status->string s)
  (define time
    (let* ([s   (status-elapsed s)]              ; seconds (total)
           [h   (floor (/ s seconds-in-hour))]   ; hours
           [s   (- s (* h seconds-in-hour))]     ; seconds (beyond hours)
           [m   (floor (/ s seconds-in-minute))] ; minutes
           [s   (- s (* m seconds-in-minute))]   ; seconds (beyond minutes)
           [fmt (λ ([t : Real]) (~r t #:precision 0 #:min-width 2 #:pad-string "0"))]
           [hh  (if (> h 0) `(,(fmt h)) '())]
           [mm  `(,(fmt m))]
           [ss  `(,(fmt s))])
      (string-join (append hh mm ss) ":")))
  (define percentage
    (let ([cur (status-elapsed s)]
          [tot (status-duration s)])
      (if (> (status-duration s) 0)
          (format "~a%" (~r (* 100 (/ cur tot)) #:precision 0 #:min-width 3))
          "   ~")))
  (format "(~a ~a ~a)"
          (state->string (status-state s))
          (~a time #:width 8 #:align 'right)
          percentage))

(: main (->* (#:host String #:port Integer Nonnegative-Real) () Void))
(define (main #:host host #:port port interval)
  (let loop ([c       : (Option Conn)   #f]
             [printer : (Option Thread) #f]
             [failures : Natural         0]
             [backoff  : Nonnegative-Real interval])
    (with-handlers*
      ([exn:fail?
         (λ (e)
            (when c
              (conn-close c))
            (let* ([failures (+ 1 failures)]
                   [next-backoff (+ interval backoff)]
                   [next-backoff (if (<= next-backoff 60) next-backoff 60)])
              (log-error
                "Network failure ~a. Backing off for ~a seconds. Exception: ~v"
                failures
                backoff
                e)
              (sleep backoff)
              (loop #f printer failures next-backoff)))])
      (let* ([c
               : Conn
               (if c c (conn-open host port))]
             [status
               : String
               (status->string (msg->status (send/recv c 'status)))]
             [printer
               : Thread
               (begin
                 (when printer (kill-thread printer))
                 (thread (λ () (print/retry status))))])
        (sleep interval)
        (loop c printer 0 interval))))
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
