((session-name
   ; string?
   . "pista")
 (base-dir
   ; string?
   . "/home/siraaj/.pista-in")
 (sensors
   ; ((sensor-executable sensor-params width timeout) ...)
   ;
   ; sensor-executable : string?
   ; sensor-params     : string?
   ; width             : positive-integer?
   ; timeout           : (or/c -1 nonnegative-integer?)
   . (("pista-sensor-upower"                        11    120)
      ("pista-sensor-wifi"        "wlp4s0 5"         8     10)
      ("pista-sensor-bluetooth"                      9     10)
      ("pista-sensor-backlight"                     10     -1)
      ("pista-sensor-volume"                         8     -1)
      ("pista-sensor-mpd"                           17      5)
      ("pista-sensor-weather-gov" "-n -i 1800 KJFK"  8   1800)
      ("pista-sensor-time"                          21      2))))
