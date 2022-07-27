#! /usr/bin/env racket
;;; vim:ft=racket:
#lang racket

(struct/contract Sensor
                 ([executable string?]
                  [parameters string?]
                  [width      positive-integer?]
                  [timeout    (or/c -1 nonnegative-integer?)])
                 #:transparent)

(struct/contract Cfg
                 ([session    string?]
                  [sock       path-string?]
                  [fifo-dir   path-string?]
                  [pista-opts (listof string?)]
                  [sensors    (listof Sensor?)])
                 #:transparent)

(define cfg (make-parameter #f))

(define/contract (file->cfg path)
  (-> path-string? Cfg?)
  (define data (with-input-from-file path (thunk (read))))
  (define session (dict-ref data 'session-name))
  (define sock (dict-ref data 'session-sock))
  (define fifo-dir (dict-ref data 'fifo-dir))
  (define pista-opts (dict-ref data 'pista-opts))
  (define sensors
    (map (match-lambda [(list e p w t) (Sensor e p w t)])
         (dict-ref data 'sensors)))
  (Cfg session sock fifo-dir pista-opts sensors))

(define (tmux fmt args)
  (format "tmux -S ~s ~a" (Cfg-sock (cfg)) (format fmt args)))

(define/contract (start)
  (-> void?)
  (define c (cfg))
  (define (sensor->pista-arg s)
    (define fifo (path->string (build-path (Cfg-fifo-dir c) (Sensor-executable s))))
    (format "~s ~s ~s" fifo (Sensor-width s) (Sensor-timeout s)))
  (define cmd-pista
    (string-join (append '("pista")
                         (Cfg-pista-opts c)
                         (map sensor->pista-arg (Cfg-sensors c)))))
  (define cmd-session
    (tmux (format "new-session -d -s ~s" (Cfg-session c))))
  (define cmds-windows
    (append*
      (for/list ([i (in-naturals)]
                 [s (Cfg-sensors c)])
                (define win i)
                (define exe (Sensor-executable s))
                (define arg (Sensor-parameters s))
                (define fifo (path->string (build-path (Cfg-fifo-dir c) exe)))
                (define cmd-sensor
                  (format "~s ~s > ~s; notify-send -u critical 'pista-sensor exited!' \"~a\n$?\""
                          exe arg fifo exe))
                (list (format "rm -f ~a" fifo)
                      (format "mkfifo ~a" fifo)
                      (tmux "new-window -t ~s" (Cfg-session c))
                      (tmux "send-keys -t ~s:~s ~s ENTER" (Cfg-session c) win cmd-sensor)))))
  (define commands (append (list cmd-session) cmds-windows (list cmd-pista)))
  (for-each system commands)
  ;(for-each displayln commands)
  )

(define (stop)
  (raise 'stop-not-implemented))

(define (restart)
  (raise 'restart-not-implemented))

(define (attach)
  (raise 'attach-not-implemented))

(module+ main
  (let ([cmd restart]
        [cfg-file (expand-user-path "~/.pista-sensors-conf.rktd")])
    (command-line
      #:once-each
      [("-c" "--conf")
       path-to-config-file "Path to configuration file."
       (invariant-assertion path-string? path-to-config-file)
       (set! cfg-file path-to-config-file)]
      #:args ([command "restart"])
      (set! cmd (match command
                  ["start"   start]
                  ["stop"    stop]
                  ["restart" restart]
                  ["attach"  attach]
                  [_ (eprintf "Error. Unknown command: ~a~n" command)
                     (exit 1)])))
    (invariant-assertion file-exists? cfg-file)
    (cfg (file->cfg cfg-file))
    (make-directory* (Cfg-fifo-dir (cfg)))
    (current-directory (Cfg-fifo-dir (cfg)))
    (cmd)))
