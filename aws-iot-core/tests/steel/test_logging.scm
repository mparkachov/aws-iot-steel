;; Steel test for logging functionality
;; This test verifies logging operations at different levels

(begin
  (log-info "=== Starting Logging Test ===")
  
  ;; Test basic logging at different levels
  (log-info "Testing basic logging levels...")
  
  (log-debug "This is a debug message")
  (log-info "This is an info message")
  (log-warn "This is a warning message")
  (log-error "This is an error message")
  
  (log-info "✓ Basic logging levels: PASSED")
  
  ;; Test logging with formatted messages
  (log-info "Testing formatted logging...")
  
  (let ((device-id "test-device-123")
        (temperature 25.5)
        (count 42))
    (log-info (format "Device ~a temperature: ~a°C" device-id temperature))
    (log-debug (format "Processing item ~a of ~a" count 100))
    (log-warn (format "Temperature ~a exceeds threshold" temperature)))
  
  (log-info "✓ Formatted logging: PASSED")
  
  ;; Test logging with special characters
  (log-info "Testing logging with special characters...")
  
  (log-info "Message with \"quotes\" and 'apostrophes'")
  (log-info "Message with newlines:\nLine 1\nLine 2")
  (log-info "Message with unicode: 🚀 ✓ ⚠️ ❌")
  (log-info "Message with symbols: @#$%^&*()_+-=[]{}|;:,.<>?")
  
  (log-info "✓ Special characters logging: PASSED")
  
  ;; Test logging with empty and null messages
  (log-info "Testing edge case logging...")
  
  (log-info "")  ; Empty message
  (log-debug "   ")  ; Whitespace only
  (log-warn (format "~a" ""))  ; Formatted empty string
  
  (log-info "✓ Edge case logging: PASSED")
  
  ;; Test logging during operations
  (log-info "Testing logging during operations...")
  
  (log-debug "Starting LED operation sequence")
  (led-on)
  (log-info "LED turned on")
  (sleep 0.05)
  (log-debug "Sleep completed")
  (led-off)
  (log-info "LED turned off")
  
  (log-info "✓ Logging during operations: PASSED")
  
  ;; Test high-frequency logging
  (log-info "Testing high-frequency logging...")
  
  (define (rapid-log-test count)
    (if (> count 0)
        (begin
          (log-debug (format "Rapid log message ~a" count))
          (rapid-log-test (- count 1)))
        #t))
  
  (rapid-log-test 20)
  (log-info "✓ High-frequency logging: PASSED")
  
  ;; Test logging with device info
  (log-info "Testing logging with device info...")
  
  (let ((info (device-info)))
    (log-info (format "Device ID: ~a" (get info 'device-id)))
    (log-info (format "Platform: ~a" (get info 'platform)))
    (log-info (format "Version: ~a" (get info 'version))))
  
  (let ((memory (memory-info)))
    (log-debug (format "Memory - Total: ~a, Free: ~a, Used: ~a"
                      (get memory 'total-bytes)
                      (get memory 'free-bytes)
                      (get memory 'used-bytes))))
  
  (log-info "✓ Logging with device info: PASSED")
  
  ;; Test logging in loops and conditions
  (log-info "Testing logging in control structures...")
  
  (define (conditional-log-test value)
    (cond
      [(< value 10) (log-debug (format "Value ~a is small" value))]
      [(< value 50) (log-info (format "Value ~a is medium" value))]
      [else (log-warn (format "Value ~a is large" value))]))
  
  (conditional-log-test 5)
  (conditional-log-test 25)
  (conditional-log-test 75)
  
  (define (loop-log-test count)
    (if (> count 0)
        (begin
          (if (= (modulo count 5) 0)
              (log-info (format "Loop iteration ~a (milestone)" count))
              (log-debug (format "Loop iteration ~a" count)))
          (loop-log-test (- count 1)))
        #t))
  
  (loop-log-test 10)
  (log-info "✓ Logging in control structures: PASSED")
  
  ;; Test logging with error simulation
  (log-info "Testing logging with error scenarios...")
  
  (log-warn "Simulating warning condition")
  (log-error "Simulating error condition")
  (log-info "System recovered from simulated error")
  
  ;; Test logging with nested operations
  (define (nested-operation-with-logging depth)
    (log-debug (format "Entering nested operation at depth ~a" depth))
    (if (> depth 0)
        (begin
          (sleep 0.01)
          (nested-operation-with-logging (- depth 1)))
        (log-debug "Reached maximum nesting depth"))
    (log-debug (format "Exiting nested operation at depth ~a" depth)))
  
  (nested-operation-with-logging 3)
  (log-info "✓ Logging with error scenarios: PASSED")
  
  ;; Test logging performance impact
  (log-info "Testing logging performance impact...")
  
  (let ((start-time (current-time)))
    ;; Perform operations with logging
    (define (operations-with-logging count)
      (if (> count 0)
          (begin
            (log-debug (format "Operation ~a starting" count))
            (led-on)
            (sleep 0.001)
            (led-off)
            (log-debug (format "Operation ~a completed" count))
            (operations-with-logging (- count 1)))
          #t))
    
    (operations-with-logging 5)
    (let ((end-time (current-time)))
      (let ((elapsed (- end-time start-time)))
        (log-info (format "Operations with logging completed in ~a seconds" elapsed))
        (if (< elapsed 1.0)  ; Should complete reasonably quickly
            (log-info "✓ Logging performance impact: PASSED")
            (begin
              (log-error "✗ Logging performance impact: FAILED - too slow")
              (error "Logging caused significant performance impact"))))))
  
  ;; Test logging with concurrent operations
  (log-info "Testing logging with concurrent-style operations...")
  
  (define (interleaved-operations count)
    (if (> count 0)
        (begin
          (log-debug (format "Starting operation set ~a" count))
          (led-on)
          (log-debug "LED on")
          (let ((info (device-info)))
            (log-debug (format "Device info retrieved: ~a" (get info 'device-id))))
          (sleep 0.01)
          (log-debug "Sleep completed")
          (led-off)
          (log-debug "LED off")
          (log-info (format "Operation set ~a completed" count))
          (interleaved-operations (- count 1)))
        #t))
  
  (interleaved-operations 3)
  (log-info "✓ Logging with concurrent-style operations: PASSED")
  
  (log-info "=== Logging Test Completed Successfully ===")
  #t)