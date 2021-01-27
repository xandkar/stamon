#lang racket

(require racket/logging)

(provide sensor:print/retry
         sensor:logger-start)

(define/contract (sensor:print/retry payload [init-backoff 1])
  (-> string? void?)
  ; We expect occasional broken pipes:
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

(define/contract (sensor:logger-start level)
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
