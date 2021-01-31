#lang racket

(require racket/logging)

(require libnotify)

(provide print/retry
         logger-start
         notify)

(define/contract (print/retry payload [init-backoff 1])
  (-> string? void?)
  ; Q: Why do we expect print failures?
  ; A: We expect our stdout to be redirected to a FIFO, which is then read by
  ; pista, which closes the pipe between message reads. We therefore expect
  ; occasional broken pipes.
  ;   UPower monitor is especially prone to encounter broken pipes, because
  ; often the updates will come in bursts from UPower and pista will close the
  ; pipe after reading the first message, which maybe while the subsequent ones
  ; from the burst are still being written.
  ;
  ; Perhaps pista should allow more than a single message before pipe closure?
  (let retry ([backoff init-backoff])
    (with-handlers
      ([exn?
         (λ (e)
            (log-error
              "Print failure. Retrying in ~a seconds. Exception: ~v" backoff e)
            (sleep backoff)
            (define jitter (random))
            (retry (+ jitter (* 2 backoff))))])
      (displayln payload)
      (flush-output))))

(define/contract (logger-start level)
  (-> log-level/c void?)
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

(define/contract (notify summary body urgency)
  (-> string? string? (or/c 'critical 'normal 'low) void?)
  (send (new notification%
             [summary summary]
             [body    body]
             [urgency urgency])
        show))
