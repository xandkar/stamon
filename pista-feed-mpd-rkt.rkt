; MPD API reference:
; https://www.musicpd.org/doc/html/protocol.html
;
#lang typed/racket

(require typed/racket/date)

(require "feed.rkt")

(module+ test
  (require typed/rackunit))

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
  (close-output-port (conn-op c))
  (log-debug "Disconnected."))

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

(: status->percentage-string (-> Status String))
(define (status->percentage-string s)
  (define cur (status-elapsed s))
  (define tot (status-duration s))
  (cond [(equal? 'stop (status-state s))
         "---"]
        [(not (> tot 0))  ; streaming
         "~~~"]
        [else
          (~r (* 100 (/ cur tot)) #:precision 0)]))

(: seconds->hms (-> Nonnegative-Real (List Real Real Real)))
(define (seconds->hms s)
  (let* ([s (floor s)]
         [h (floor (/ s seconds-in-hour))]   ; hours
         [s (- s (* h seconds-in-hour))]     ; seconds (beyond hours)
         [m (floor (/ s seconds-in-minute))] ; minutes
         [s (- s (* m seconds-in-minute))])  ; seconds (beyond minutes)
    (list h m s)))

(module+ test
  (check-equal? (seconds->hms        0)  (list 0  0  0))
  (check-equal? (seconds->hms        1)  (list 0  0  1))
  (check-equal? (seconds->hms        5)  (list 0  0  5))
  (check-equal? (seconds->hms       10)  (list 0  0 10))
  (check-equal? (seconds->hms (* 31 60)) (list 0 31  0))
  (check-equal? (seconds->hms       60)  (list 0  1  0))
  (check-equal? (seconds->hms      120)  (list 0  2  0))
  (check-equal? (seconds->hms     3600)  (list 1  0  0))
  (check-equal? (seconds->hms     7200)  (list 2  0  0))
  (check-equal? (seconds->hms     7260)  (list 2  1  0))
  (check-equal? (seconds->hms     7269)  (list 2  1  9))
  )

(: status->time-string (-> Status String))
(define (status->time-string s)
  (match (status-state s)
    ['stop
     "--:--"]
    [_
      (match (seconds->hms (status-elapsed s))
        [(list h m s)
         (let* ([fmt (λ ([t : Real]) (~r t #:precision 0 #:min-width 2 #:pad-string "0"))]
                [hh  (if (> h 0) `(,(fmt h)) '())]
                [mm  `(,(fmt m))]
                [ss  `(,(fmt s))])
           (string-join (append hh mm ss) ":"))])]))

(module+ test
  (check-equal? (status->time-string (status 'stop 0 0)) "--:--")
  (check-equal? (status->time-string (status 'stop 10 5)) "--:--")
  (check-equal? (status->time-string (status 'play 0 0)) "00:00")
  (check-equal? (status->time-string (status 'play 60 0)) "01:00")
  (check-equal? (status->time-string (status 'play 60 100)) "01:00") ; duration noop
  (check-equal? (status->time-string (status 'play (* 32 60) 0)) "32:00")
  (check-equal? (status->time-string (status 'play 59.5 0)) "00:59")
  (check-equal? (status->time-string (status 'play 7269 0)) "02:01:09")
  )

(: log-memory-usage (-> (Option Output-Port) Void))
(define (log-memory-usage mem-log)
  ; TODO Handle IO errors
  (when mem-log
    (displayln (format "~a ~a"
                       (date->seconds (current-date))
                       (current-memory-use))
               mem-log)
    (flush-output mem-log)))

(: status->string (-> Status String))
(define (status->string s)
  (format "~a ~a ~a%"
          (state->string (status-state s))
          (~a (status->time-string s)       #:width 8 #:align 'right)
          (~a (status->percentage-string s) #:width 3 #:align 'right)))

(: main (->* (
              #:host     String
              #:port     Integer
              #:interval Nonnegative-Real
              #:mem-log  (Option Output-Port))
             ()
             Void))
(define (main #:host host
              #:port port
              #:interval interval
              #:mem-log mem-log)
  (define printer : (Option Thread) #f)
  (define c       : (Option Conn)   #f)
  (let retry ([failures : Natural          0]
              [backoff  : Nonnegative-Real interval])
    (log-memory-usage mem-log)
    (with-handlers*
      ([exn:fail?
         (λ (e)
            (when c (conn-close (assert c)))
            (let* ([failures (+ 1 failures)]
                   [next-backoff (+ interval backoff)]
                   [next-backoff (if (<= next-backoff 60) next-backoff 60)])
              (log-debug
                "Network failure ~a. Backing off for ~a seconds. Exception: ~v"
                failures
                backoff
                e)
              (sleep backoff)
              (retry failures next-backoff)))])
      (set! c (conn-open host port))
      (let poll ([c : Conn (assert c)])
        (log-memory-usage mem-log)
        (let ([s : String (status->string (msg->status (send/recv c 'status)))])
          (when printer (kill-thread (assert printer)))
          (set! printer (thread (λ () (print/retry s)))))
        (sleep interval)
        (poll c))))
  (flush-output (current-error-port)))

(module+ main
  (define opt-host "localhost")
  (define opt-port 6600)
  (define opt-log-level : Log-Level 'info)
  (define opt-interval-seconds : Nonnegative-Real 1)
  (define opt-mem-log : (Option Path-String) #f)
  (command-line
    #:once-each
    [("-d" "--debug")
     "Enable debug logging"
     (set! opt-log-level 'debug)]
    [("-i" "--interval")
     i "Poll interval"
     (set! opt-interval-seconds
           (cast (string->number (cast i String)) Nonnegative-Real))]
    [("-m" "--mem-log")
     m "Path to a file to which memory usage will be logged"
     (set! opt-mem-log (string->path (cast m String)))])
  (logger-start opt-log-level)
  (main
    #:host opt-host
    #:port opt-port
    #:interval opt-interval-seconds
    #:mem-log (if opt-mem-log
                  (open-output-file (assert opt-mem-log) #:exists 'append)
                  #f)))
