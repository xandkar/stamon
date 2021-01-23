#! /usr/bin/env racket

#lang racket

(require racket/date)

(require openweather)

(struct interval
        (normal error-init error-curr)
        #:transparent)

(define (interval-reset i)
  (struct-copy interval i [error-curr (interval-error-init i)]))

(define (interval-increase i)
  (struct-copy interval i [error-curr (* 2 (interval-error-curr i))]))

(define/contract (main api-key zip-code interval)
  (-> string? string? interval? void?)
  (log-info "Starting main loop with:~n~a"
            (pretty-format
              `([api-key  ,api-key]
                [zip-code ,zip-code]
                ,interval)))
  (let loop ([i interval])
    (match (OpenWeatherMap-temp (owm-invoke api-key zip-code))
      [#f
        (log-error "Data fetch failed.")
        (sleep (interval-error-curr i))
        (loop (interval-increase i))]
      [temp
        (with-handlers
          ; Expecting broken pipes
          ([exn:fail:filesystem:errno? (λ (e) (log-error "Print failed: ~v" e))])
          (printf "(~a°F)\n" (~r temp #:min-width 3 #:precision 0))
          (flush-output))
        (sleep (interval-normal i))
        (loop (interval-reset i))])))

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

(module+ main
  (date-display-format 'rfc2822)
  (define one-minute 60)
  (define opt-interval (* 30 one-minute))
  (define opt-backoff one-minute)
  (define opt-log-level 'info)
  (command-line #:once-each
                [("-d" "--debug")
                 "Enable debug logging"
                 (set! opt-log-level 'debug)]
                [("-i" "--interval")
                 i "Refresh interval."
                 (set! opt-interval (string->number i))]
                [("-b" "--backoff")
                 b "Initial retry backoff period (subsequently doubled)."
                 (set! opt-backoff (string->number b))]
                #:args
                (api-key zip-code)
                (start-logger opt-log-level)
                (main api-key zip-code
                      (interval opt-interval
                                opt-backoff
                                opt-backoff))))
