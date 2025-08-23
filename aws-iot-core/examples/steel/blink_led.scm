;; LED Blinking Example
;; This example demonstrates basic LED control with timing

(define (blink-led times delay)
  "Blink the LED a specified number of times with given delay"
  (begin
    (log-info (string-append "Starting LED blink sequence: " 
                            (->string times) " times with " 
                            (->string delay) "s delay"))
    
    (define (blink-loop remaining)
      (if (> remaining 0)
          (begin
            (log-info (string-append "Blink " (->string (- times remaining -1))))
            (led-on)
            (sleep delay)
            (led-off)
            (sleep delay)
            (blink-loop (- remaining 1)))
          (log-info "Blink sequence completed")))
    
    (blink-loop times)))

(define (main)
  "Main function - blink LED 5 times"
  (begin
    (log-info "=== LED Blink Example ===")
    (blink-led 5 0.5)
    (log-info "Example completed successfully")))

;; Run the example
(main)