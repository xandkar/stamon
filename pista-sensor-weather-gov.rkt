#! /usr/bin/env racket

#lang racket

(require net/http-client)
(require racket/date)
(require xml)
(require xml/path)

(struct interval (normal error-init error-curr))

(define (interval-reset i)
  (struct-copy interval i [error-curr (interval-error-init i)]))

(define (interval-increase i)
  (struct-copy interval i [error-curr (* 2 (interval-error-curr i))]))

(define data?
  (listof (cons/c symbol? (or/c string? number?))))

(define/contract (xexpr->data x)
  (-> xexpr? data?)
  (define (str path [default #f])
    (let ([val (se-path* (append '(current_observation) path) x)])
      (cons (car path) (if val val default))))
  (define (num path)
    ; TODO better handling of missing values than defaulting to 0?
    (match (str path "0")
      [`(,k . ,v) (cons k (string->number v))]))
  (list
    (str  '(station_id))
    (str  '(location))
    (str  '(observation_time_rfc822))
    (str  '(suggested_pickup))
    (num  '(suggested_pickup_period))
    (str  '(weather))
    (str  '(temperature_string))
    (num  '(temp_f))
    (num  '(temp_c))
    (num  '(relative_humidity))
    (str  '(wind_string))
    (str  '(wind_dir))
    (num  '(wind_mph))
    (str  '(pressure_string))
    (str  '(dewpoint_string))
    (num  '(visibility_mi))
    ))

(define data<-port
  (compose xexpr->data string->xexpr port->string))

(define/contract (data-fetch station-id)
  (-> string? (or/c (cons/c 'ok data?)
                    (cons/c 'error number?)))
  (define-values (status-line headers data-port)
    (http-sendrecv
      "api.weather.gov"
      (format "/stations/~a/observations/latest?require_qc=false" station-id)
      #:ssl? #t
      #:headers '("accept: application/vnd.noaa.obs+xml")))
  (log-debug "headers ~v" headers)
  (log-debug "status-line: ~v" status-line)
  (define status (string-split (bytes->string/utf-8 status-line)))
  (log-debug "status: ~v" status)
  (define status-code (string->number (second status)))
  (log-debug "status-code: ~v" status-code)
  (if (= 200 status-code)
      (cons 'ok (data<-port data-port))
      (cons 'error status-code)))

(define (data-notify data)
  (define (get key) (dict-ref data key))
  (define n->s number->string)
  (local-require libnotify)
  (send (new notification%
             [summary (format "Weather updated")]
             [body (string-append
                     "\n"
                     (get 'location) "\n"
                     "\n"
                     "observation : " (get 'observation_time_rfc822) "\n"
                     "current     : " (date->string (current-date) #t) "\n"
                     "\n"
                     "weather     : " (get 'weather) "\n"
                     "temperature : "       (get 'temperature_string) "\n"
                     "humidity    : " (n->s (get 'relative_humidity)) "\n"
                     "wind        : "       (get 'wind_string) "\n"
                     "pressure    : "       (get 'pressure_string) "\n"
                     "dewpoint    : "       (get 'dewpoint_string) "\n"
                     "visibility  : " (n->s (get 'visibility_mi)) " miles\n"
                     )]

             [urgency 'low])
        show))

(define/contract (data-print data)
  (-> data? void?)
  (log-info "Known fetched data:~n~a" (pretty-format data))
  (with-handlers
    ; Expecting broken pipes
    ; TODO Retry failed prints
    ([exn:fail:filesystem:errno? (λ (e) (log-error "Print failed: ~v" e))])
    (printf "(~a°F)\n" (~r (dict-ref data 'temp_f)
                           #:min-width 3
                           #:precision 0))
    (flush-output)))

(define/contract (loop station-id interval notify?)
  (-> string? interval? boolean? void?)
  (let loop ([i interval])
    (match (data-fetch station-id)
      [(cons 'error status-code)
       (log-error "Data fetch failed with ~a" status-code)
       (sleep (interval-error-curr i))
       (loop (interval-increase i))]
      [(cons 'ok data)
       (data-print data)
       (when notify?
         (data-notify data))
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
  (define opt-notify #f)
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
                [("-n" "--notify")
                 "Enable notifications"
                 (set! opt-notify #t)]
                #:args
                (station-id)
                (start-logger opt-log-level)
                (loop station-id
                      (interval opt-interval
                                opt-backoff
                                opt-backoff)
                      opt-notify)))

; API docs at https://www.weather.gov/documentation/services-web-api

; Example raw data for KJFK:
;
;    <?xml version="1.0" encoding="UTF-8"?>
;    <current_observation version="1.0" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://www.weather.gov/view/current_observation.xsd">
;     <credit>NOAA's National Weather Service</credit>
;     <credit_URL>http://weather.gov/</credit_URL>
;     <image>
;      <url>http://weather.gov/images/xml_logo.gif</url>
;      <title>NOAA's National Weather Service</title>
;      <link>http://weather.gov/</link>
;     </image>
;     <suggested_pickup>15 minutes after the hour</suggested_pickup>
;     <suggested_pickup_period>60</suggested_pickup_period>
;     <location>New York, Kennedy International Airport, NY</location>
;     <station_id>KJFK</station_id>
;     <latitude>40.63915</latitude>
;     <longitude>-73.76393</longitude>
;     <observation_time>Last Updated on Jan 13 2021, 10:51 am GMT+0000</observation_time>
;     <observation_time_rfc822>Wed, 13 Jan 21 10:51:00 +0000</observation_time_rfc822>
;     <weather>Cloudy</weather>
;     <temperature_string>34 F (1.1 C)</temperature_string>
;     <temp_f>34</temp_f>
;     <temp_c>1.1</temp_c>
;     <relative_humidity>72</relative_humidity>
;     <wind_string>N at 0 MPH (0 KT)</wind_string>
;     <wind_dir>N</wind_dir>
;     <wind_degrees>0</wind_degrees>
;     <wind_mph>0</wind_mph>
;     <wind_kt>0</wind_kt>
;     <pressure_string>1018.6 mb</pressure_string>
;     <pressure_mb>1018.6</pressure_mb>
;     <pressure_in>30.08</pressure_in>
;     <dewpoint_string>26.1 F (-3.3 C)</dewpoint_string>
;     <dewpoint_f>26.1</dewpoint_f>
;     <dewpoint_c>-3.3</dewpoint_c>
;     <visibility_mi>10</visibility_mi>
;     <icon_url_base>https://api.weather.gov/icons/land</icon_url_base>
;     <two_day_history_url>https://forecast-v3.weather.gov/obs/KJFK/history</two_day_history_url>
;     <icon_url_name>night</icon_url_name>
;     <ob_url>https://www.weather.gov/data/METAR/KJFK.1.txt</ob_url>
;     <disclaimer_url>https://weather.gov/disclaimer.html</disclaimer_url>
;     <copyright_url>https://weather.gov/disclaimer.html</copyright_url>
;     <privacy_policy_url>https://weather.gov/notice.html</privacy_policy_url>
;    </current_observation>
