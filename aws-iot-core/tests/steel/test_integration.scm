;; Steel integration test
;; This test combines multiple functionalities to test real-world scenarios

(define (test-led-with-system-info)
  "Test LED control combined with system information"
  (begin
    (log-info "Starting LED with system info integration test")
    
    ;; Get initial system state
    (let ((initial-memory (memory-info))
          (device (device-info)))
      
      (log-info "=== Initial System State ===")
      (log-info (string-append "Device: " (->string device)))
      (log-info (string-append "Memory: " (->string initial-memory)))
      
      ;; Perform LED operations
      (log-info "Performing LED operations...")
      (led-on)
      (sleep 0.001)
      (let ((led-state-on (led-state)))
        (if led-state-on
            (log-info "✓ LED is on as expected")
            (error "LED should be on")))
      
      (led-off)
      (sleep 0.001)
      (let ((led-state-off (led-state)))
        (if (not led-state-off)
            (log-info "✓ LED is off as expected")
            (error "LED should be off")))
      
      ;; Check system state after operations
      (let ((final-memory (memory-info))
            (final-uptime (uptime)))
        (log-info "=== Final System State ===")
        (log-info (string-append "Memory: " (->string final-memory)))
        (log-info (string-append "Uptime: " (->string final-uptime) " seconds"))))
    
    (log-info "LED with system info integration test completed successfully")
    #t))

(define (test-error-recovery)
  "Test error handling and recovery"
  (begin
    (log-info "Starting error recovery test")
    
    ;; Test recovery from sleep error
    (define sleep-error-handled
      (call/cc
        (lambda (escape)
          (with-exception-handler
            (lambda (e)
              (log-info "✓ Sleep error handled gracefully")
              (escape #t))
            (lambda ()
              (sleep -1)
              (escape #f))))))
    
    ;; Continue with normal operations after error
    (if sleep-error-handled
        (begin
          (log-info "Continuing with normal operations after error...")
          (led-on)
          (sleep 0.001)
          (led-off)
          (log-info "✓ Normal operations resumed successfully"))
        (error "Error recovery test failed"))
    
    (log-info "Error recovery test completed successfully")
    #t))

(define (test-performance-scenario)
  "Test performance with multiple operations"
  (begin
    (log-info "Starting performance scenario test")
    
    ;; Simulate a monitoring loop
    (define (monitoring-loop iterations)
      (if (> iterations 0)
          (begin
            ;; Check system status
            (let ((memory (memory-info))
                  (uptime (uptime)))
              
              ;; Simulate some processing
              (led-on)
              (sleep 0.001)
              (led-off)
              
              ;; Log status every 5 iterations
              (if (= (modulo iterations 5) 0)
                  (log-info (string-append "Monitoring iteration " (->string iterations))))
              
              (monitoring-loop (- iterations 1))))
          #t))
    
    (log-info "Running monitoring simulation...")
    (monitoring-loop 10)
    (log-info "✓ Monitoring simulation completed")
    
    (log-info "Performance scenario test completed successfully")
    #t))

(define (run-integration-tests)
  "Run all integration tests"
  (begin
    (log-info "=== Running Steel Integration Tests ===")
    (test-led-with-system-info)
    (test-error-recovery)
    (test-performance-scenario)
    (log-info "=== All integration tests passed ===")
    #t))

;; Run the tests
(run-integration-tests)