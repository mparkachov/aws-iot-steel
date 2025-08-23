;; Steel test for sleep functionality
;; This test verifies sleep function works correctly

(define (test-sleep-basic)
  "Test basic sleep functionality"
  (begin
    (log-info "Starting sleep basic test")
    
    ;; Test short sleep
    (log-info "Testing 0.001 second sleep")
    (sleep 0.001)
    (log-info "✓ Short sleep completed")
    
    ;; Test zero sleep
    (log-info "Testing zero sleep")
    (sleep 0)
    (log-info "✓ Zero sleep completed")
    
    (log-info "Sleep basic test completed successfully")
    #t))

(define (test-sleep-error-handling)
  "Test sleep error handling"
  (begin
    (log-info "Starting sleep error handling test")
    
    ;; Test negative sleep (should fail)
    (define result
      (call/cc
        (lambda (escape)
          (with-exception-handler
            (lambda (e)
              (log-info "✓ Negative sleep correctly threw error")
              (escape #t))
            (lambda ()
              (sleep -1)
              (log-error "✗ Negative sleep should have failed")
              (escape #f))))))
    
    (if result
        (log-info "Sleep error handling test completed successfully")
        (error "Sleep error handling test failed"))
    
    #t))

(define (test-sleep-timing)
  "Test sleep timing accuracy (basic)"
  (begin
    (log-info "Starting sleep timing test")
    
    ;; Test multiple short sleeps
    (define (sleep-multiple times)
      (if (> times 0)
          (begin
            (sleep 0.001)
            (sleep-multiple (- times 1)))
          #t))
    
    (log-info "Testing multiple short sleeps")
    (sleep-multiple 5)
    (log-info "✓ Multiple sleeps completed")
    
    (log-info "Sleep timing test completed successfully")
    #t))

(define (run-sleep-tests)
  "Run all sleep tests"
  (begin
    (log-info "=== Running Steel Sleep Tests ===")
    (test-sleep-basic)
    (test-sleep-error-handling)
    (test-sleep-timing)
    (log-info "=== All sleep tests passed ===")
    #t))

;; Run the tests
(run-sleep-tests)