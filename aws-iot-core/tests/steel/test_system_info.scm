;; Steel test for system information functionality
;; This test verifies device info, memory info, and uptime functions

(define (test-device-info)
  "Test device information retrieval"
  (begin
    (log-info "Starting device info test")
    
    (let ((info (device-info)))
      (if (list? info)
          (begin
            (log-info "✓ Device info returned a list")
            (log-info (string-append "Device info: " (->string info))))
          (begin
            (log-error "✗ Device info should return a list")
            (error "Device info test failed"))))
    
    (log-info "Device info test completed successfully")
    #t))

(define (test-memory-info)
  "Test memory information retrieval"
  (begin
    (log-info "Starting memory info test")
    
    (let ((info (memory-info)))
      (if (list? info)
          (begin
            (log-info "✓ Memory info returned a list")
            (log-info (string-append "Memory info: " (->string info))))
          (begin
            (log-error "✗ Memory info should return a list")
            (error "Memory info test failed"))))
    
    (log-info "Memory info test completed successfully")
    #t))

(define (test-uptime)
  "Test uptime retrieval"
  (begin
    (log-info "Starting uptime test")
    
    (let ((up (uptime)))
      (if (number? up)
          (begin
            (log-info "✓ Uptime returned a number")
            (log-info (string-append "Uptime: " (->string up) " seconds")))
          (begin
            (log-error "✗ Uptime should return a number")
            (error "Uptime test failed"))))
    
    (log-info "Uptime test completed successfully")
    #t))

(define (test-system-info-integration)
  "Test system information integration"
  (begin
    (log-info "Starting system info integration test")
    
    ;; Get all system information
    (let ((device (device-info))
          (memory (memory-info))
          (up (uptime)))
      
      (log-info "=== System Information Summary ===")
      (log-info (string-append "Device: " (->string device)))
      (log-info (string-append "Memory: " (->string memory)))
      (log-info (string-append "Uptime: " (->string up) " seconds"))
      (log-info "=== End Summary ==="))
    
    (log-info "System info integration test completed successfully")
    #t))

(define (run-system-info-tests)
  "Run all system information tests"
  (begin
    (log-info "=== Running Steel System Info Tests ===")
    (test-device-info)
    (test-memory-info)
    (test-uptime)
    (test-system-info-integration)
    (log-info "=== All system info tests passed ===")
    #t))

;; Run the tests
(run-system-info-tests)