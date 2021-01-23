; vim: filetype=racket
#lang info

(define collection
  "pista-sensors")

(define pkg-desc
  "pista sensors")

(define version
  "0.0.0")

(define pkg-authors
  '("Siraaj Khandkar <siraaj@khandkar.net>"))

(define deps
  '("base"
    "libnotify"
    "openweather"))

(define racket-launcher-names
  '("pista-sensor-openweather"
    "pista-sensor-upower"
    "pista-sensor-weather-gov"))

(define racket-launcher-libraries
  '("pista-sensor-openweather.rkt"
    "pista-sensor-upower.rkt"
    "pista-sensor-weather-gov.rkt"))
