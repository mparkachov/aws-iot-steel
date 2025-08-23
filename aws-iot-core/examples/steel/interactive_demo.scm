;; Interactive Demo Example
;; This example demonstrates various Steel capabilities in an interactive way

(define (demo-led-patterns)
  "Demonstrate different LED patterns"
  (begin
    (log-info "=== LED Pattern Demo ===")
    
    ;; Pattern 1: Fast blink
    (log-info "Pattern 1: Fast blink (5 times)")
    (define (fast-blink count)
      (if (> count 0)
          (begin
            (led-on)
            (sleep 0.1)
            (led-off)
            (sleep 0.1)
            (fast-blink (- count 1)))
          #t))
    (fast-blink 5)
    
    (sleep 1)
    
    ;; Pattern 2: Slow pulse
    (log-info "Pattern 2: Slow pulse (3 times)")
    (define (slow-pulse count)
      (if (> count 0)
          (begin
            (led-on)
            (sleep 0.8)
            (led-off)
            (sleep 0.8)
            (slow-pulse (- count 1)))
          #t))
    (slow-pulse 3)
    
    (sleep 1)
    
    ;; Pattern 3: Morse code SOS
    (log-info "Pattern 3: Morse code SOS")
    (define (dot)
      (begin (led-on) (sleep 0.2) (led-off) (sleep 0.2)))
    (define (dash)
      (begin (led-on) (sleep 0.6) (led-off) (sleep 0.2)))
    (define (letter-gap)
      (sleep 0.6))
    
    ;; S (dot dot dot)
    (dot) (dot) (dot) (letter-gap)
    ;; O (dash dash dash)
    (dash) (dash) (dash) (letter-gap)
    ;; S (dot dot dot)
    (dot) (dot) (dot)
    
    (log-info "LED patterns demo completed")))

(define (demo-system-monitoring)
  "Demonstrate system monitoring capabilities"
  (begin
    (log-info "=== System Monitoring Demo ===")
    
    ;; Show initial state
    (log-info "Initial system state:")
    (let ((device (device-info))
          (memory (memory-info))
          (up (uptime)))
      (log-info (string-append "Device: " (->string device)))
      (log-info (string-append "Memory: " (->string memory)))
      (log-info (string-append "Uptime: " (->string up) " seconds")))
    
    ;; Simulate some work and monitor changes
    (log-info "Performing some work...")
    (define (do-work iterations)
      (if (> iterations 0)
          (begin
            (led-on)
            (sleep 0.05)
            (led-off)
            (sleep 0.05)
            (do-work (- iterations 1)))
          #t))
    
    (do-work 10)
    
    ;; Show final state
    (log-info "Final system state:")
    (let ((device (device-info))
          (memory (memory-info))
          (up (uptime)))
      (log-info (string-append "Device: " (->string device)))
      (log-info (string-append "Memory: " (->string memory)))
      (log-info (string-append "Uptime: " (->string up) " seconds")))
    
    (log-info "System monitoring demo completed")))

(define (demo-error-handling)
  "Demonstrate error handling capabilities"
  (begin
    (log-info "=== Error Handling Demo ===")
    
    ;; Demonstrate graceful error handling
    (log-info "Testing error handling with invalid sleep...")
    (define error-handled
      (call/cc
        (lambda (escape)
          (with-exception-handler
            (lambda (e)
              (log-warn "Caught expected error - continuing gracefully")
              (escape #t))
            (lambda ()
              (sleep -1)  ;; This should cause an error
              (escape #f))))))
    
    (if error-handled
        (log-info "✓ Error handled successfully")
        (log-error "✗ Error handling failed"))
    
    ;; Continue with normal operations
    (log-info "Continuing with normal operations...")
    (led-on)
    (sleep 0.5)
    (led-off)
    
    (log-info "Error handling demo completed")))

(define (demo-performance)
  "Demonstrate performance characteristics"
  (begin
    (log-info "=== Performance Demo ===")
    
    ;; Test rapid LED switching
    (log-info "Testing rapid LED operations...")
    (define start-time (uptime))
    
    (define (rapid-led-test count)
      (if (> count 0)
          (begin
            (led-on)
            (led-off)
            (rapid-led-test (- count 1)))
          #t))
    
    (rapid-led-test 100)
    
    (let ((end-time (uptime)))
      (log-info (string-append "Completed 100 LED operations in " 
                              (->string (- end-time start-time)) " seconds")))
    
    ;; Test logging performance
    (log-info "Testing logging performance...")
    (define log-start-time (uptime))
    
    (define (rapid-log-test count)
      (if (> count 0)
          (begin
            (log-debug (string-append "Performance test log " (->string count)))
            (rapid-log-test (- count 1)))
          #t))
    
    (rapid-log-test 50)
    
    (let ((log-end-time (uptime)))
      (log-info (string-append "Completed 50 log operations in " 
                              (->string (- log-end-time log-start-time)) " seconds")))
    
    (log-info "Performance demo completed")))

(define (main)
  "Main interactive demo function"
  (begin
    (log-info "=== Steel Interactive Demo ===")
    (log-info "This demo showcases various Steel capabilities")
    (log-info "")
    
    (demo-led-patterns)
    (sleep 2)
    
    (demo-system-monitoring)
    (sleep 2)
    
    (demo-error-handling)
    (sleep 2)
    
    (demo-performance)
    
    (log-info "")
    (log-info "=== Demo completed successfully ===")
    (log-info "Thank you for trying the Steel interactive demo!")))

;; Run the demo
(main)