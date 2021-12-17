#lang typed/racket

(require/typed racket/base
               ; TODO Why make-logger default type isn't compatible with my usage?
               [make-logger
                 (-> False False Log-Level False Logger)])

(require/typed libnotify
               [notification%
                 (Class (init [summary String]
                              [body    String]
                              [urgency (U 'critical 'normal 'low)])
                        [show (-> Void)]
                        [close (-> Void)])])

(provide print/retry
         logger-start
         notify)

(: print/retry (->* (String) (Positive-Real) Void))
(define (print/retry payload [init-backoff 1.0])
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
  (let retry ([backoff : Positive-Real init-backoff]
              [attempt : Natural       1])
    (with-handlers*
      ([exn:fail?
         (λ (e)
            (log-error
              "Print failure ~a. Retrying in ~a seconds. Exception: ~v"
              attempt
              backoff
              e)
            (sleep backoff)
            (let* ([jitter  (cast (random) Positive-Real)]
                   [backoff (+ jitter (* 2 backoff))]
                   [attempt (+ 1 attempt)])
              (retry backoff attempt))
            )])
      (displayln payload)
      (flush-output)
      (when (> attempt 1)
        (log-info "Print success after ~a attempts." attempt)))))

(: logger-start (-> Log-Level Void))
(define (logger-start level)
  ; TODO implement graceful stop, flushing before exiting
  (define logger (make-logger #f #f level #f))
  (define log-receiver (make-log-receiver logger level))
  (thread
    (λ ()
       (local-require typed/racket/date)
       (date-display-format 'iso-8601)
       (let loop ()
         (match-let ([(vector level msg _ _) (sync log-receiver)])
           (eprintf "~a [~a] ~a~n" (date->string (current-date) #t) level msg))
         (loop))))
  (current-logger logger))

(: notify (-> String String (U 'critical 'normal 'low) Void))
(define (notify summary body urgency)
  (with-handlers*
    ([exn:fail? (λ (e) (log-error "Notification failure: ~v" e))])
    (send (new notification%
               [summary summary]
               [body    body]
               [urgency urgency])
          show)))
