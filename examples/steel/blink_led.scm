;; Example: LED Blinking Program
;; This program demonstrates basic LED control with timing
(begin
  (log-info "LED Blinking Example Started")
  (define (blink-sequence count delay)
    "Blink LED count times with specified delay"
    (if (> count 0)
        (begin
          (log-info (string-append "Blink " (number->string count)))
          (led-on)
          (sleep delay)
          (led-off)
          (sleep delay)
          (blink-sequence (- count 1) delay))
        (log-info "Blink sequence completed")))
  ;; Blink 5 times with 0.5 second intervals
  (blink-sequence 5 0.5)
  (log-info "LED Blinking Example Completed")
  #t)