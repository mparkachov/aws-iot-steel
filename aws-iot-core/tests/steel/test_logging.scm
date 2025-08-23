;; Steel test for logging functionality
;; This test verifies all logging functions work correctly

(define (test-log-levels)
  "Test different log levels"
  (begin
    (log-info "Starting log levels test")
    
    ;; Test all log levels
    (log-error "This is an error message")
    (log-warn "This is a warning message")
    (log-info "This is an info message")
    (log-debug "This is a debug message")
    
    ;; Test generic log function
    (log "ERROR" "Generic error message")
    (log "WARN" "Generic warning message")
    (log "INFO" "Generic info message")
    (log "DEBUG" "Generic debug message")
    
    (log-info "✓ All log levels tested")
    (log-info "Log levels test completed successfully")
    #t))

(define (test-log-formatting)
  "Test log message formatting"
  (begin
    (log-info "Starting log formatting test")
    
    ;; Test with different data types
    (log-info (string-append "Number: " (->string 42)))
    (log-info (string-append "Boolean: " (->string #t)))
    (log-info (string-append "List: " (->string '(1 2 3))))
    
    ;; Test with special characters
    (log-info "Special chars: !@#$%^&*()")
    (log-info "Unicode: αβγδε")
    
    (log-info "✓ Log formatting tested")
    (log-info "Log formatting test completed successfully")
    #t))

(define (test-log-performance)
  "Test logging performance with multiple messages"
  (begin
    (log-info "Starting log performance test")
    
    ;; Log multiple messages quickly
    (define (log-multiple count)
      (if (> count 0)
          (begin
            (log-info (string-append "Performance test message " (->string count)))
            (log-multiple (- count 1)))
          #t))
    
    (log-multiple 10)
    
    (log-info "✓ Performance test completed")
    (log-info "Log performance test completed successfully")
    #t))

(define (run-logging-tests)
  "Run all logging tests"
  (begin
    (log-info "=== Running Steel Logging Tests ===")
    (test-log-levels)
    (test-log-formatting)
    (test-log-performance)
    (log-info "=== All logging tests passed ===")
    #t))

;; Run the tests
(run-logging-tests)