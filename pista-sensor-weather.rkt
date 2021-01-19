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

(define/contract (data-fetch weather-station-id)
  (-> string? (or/c (cons/c 'ok xexpr?)
                    (cons/c 'error number?)))
  (define-values (status-line headers data-port)
    (http-sendrecv
      "api.weather.gov"
      (string-append "/stations/"
                     weather-station-id
                     "/observations/latest?require_qc=false")
      #:ssl? #t
      #:headers '("accept: application/vnd.noaa.obs+xml")))
  (eprintf "[debug] headers ~v~n" headers)
  (eprintf "[debug] status-line: ~v~n" status-line)
  (define status (string-split (bytes->string/utf-8 status-line)))
  (eprintf "[debug] status: ~v~n" status)
  (define status-code (string->number (second status)))
  (eprintf "[debug] status-code: ~v~n" status-code)
  (if (= 200 status-code)
      (cons 'ok (string->xexpr (port->string data-port)))
      (cons 'error status-code)))

(define/contract (data-print data)
  (-> xexpr? void?)
  (define (get path) (se-path* (append '(current_observation) path) data))
  (define temp-f (string->number (get '(temp_f))))
  (define bar (make-string 25 #\-))
  (eprintf "+~a\n" bar)
  (eprintf "| ~a\n" (date->string (current-date) #t))
  (eprintf "+~a\n" bar)
  (eprintf "| station           : ~a\n" (get '(station_id)))
  (eprintf "| location          : ~a\n" (get '(location)))
  (eprintf "| timestamp         : ~a\n" (get '(observation_time_rfc822)))
  (eprintf "| suggested pickup  : ~a\n" (get '(suggested_pickup)))
  (eprintf "| suggested interval: ~a\n" (string->number (get '(suggested_pickup_period))))
  (eprintf "| temp-f            : ~a\n" temp-f)
  (eprintf "| temp-c            : ~a\n" (string->number (get '(temp_c))))
  (eprintf "| humidity          : ~a\n" (string->number (get '(relative_humidity))))
  (eprintf "| wind-dir          : ~a\n" (get '(wind_dir)))
  (eprintf "| wind-speed        : ~a\n" (string->number (get '(wind_mph))))
  (eprintf "| visibility        : ~a\n" (string->number (get '(visibility_mi))))
  (eprintf "+~a\n" bar)
  (with-handlers
    ; Expecting broken pipes
    ([exn:fail:filesystem:errno? (λ (e) (eprintf "[error] Exception when printing: ~v\n" e))])
    (printf "(~a°F)\n" (~r temp-f
                           #:min-width 3
                           #:precision 0))
    (flush-output)))

(define/contract (loop weather-station-id i)
  (-> string? interval? void?)
  (match (data-fetch weather-station-id)
         [(cons 'error status-code)
          (eprintf "[error] Data fetch failed with ~a\n" status-code)
          (sleep (interval-error-curr i))
          (loop weather-station-id (interval-increase i))]
         [(cons 'ok data)
          (data-print data)
          (sleep (interval-normal i))
          (loop weather-station-id (interval-reset i))]))

(module+ main
         (date-display-format 'rfc2822)
         (define one-minute 60)
         (define opt-interval (* 30 one-minute))
         (define opt-backoff one-minute)
         (command-line #:once-each
                       [("-i" "--interval")
                        i "Refresh interval."
                        (set! opt-interval (string->number i))]
                       [("-b" "--backoff")
                        b "Initial retry backoff period (subsequently doubled)."
                        (set! opt-backoff (string->number b))]
                       #:args
                       (weather-station-id)
                       (loop weather-station-id
                             (interval opt-interval
                                       opt-backoff
                                       opt-backoff))))

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
