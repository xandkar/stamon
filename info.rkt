; vim: filetype=racket
#lang info

(define collection
  "pista-feeds")

(define pkg-desc
  "pista feeds")

(define version
  "0.0.0")

(define pkg-authors
  '("Siraaj Khandkar <siraaj@khandkar.net>"))

(define deps
  '("base"
    "libnotify"
    "openweather"))

(define racket-launcher-libraries
  '("pista-feed-openweather.rkt"
    "pista-feed-upower.rkt"
    "pista-feed-weather-gov.rkt"))

(define racket-launcher-names
  racket-launcher-libraries)
