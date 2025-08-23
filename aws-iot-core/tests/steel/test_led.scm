;; Steel test for LED functionality
;; This test verifies LED control functions work correctly

(define (test-led-basic)
  "Test basic LED on/off functionality"
  (begin
    (log-info "Starting LED basic test")
    
    ;; Turn LED on and verify state
    (led-on)
    (let ((state (led-state)))
      (if state
          (log-info "✓ LED on test passed")
          (begin
            (log-error "✗ LED on test failed")
            (error "LED should be on but state is false"))))
    
    ;; Turn LED off and verify state
    (led-off)
    (let ((state (led-state)))
      (if (not state)
          (log-info "✓ LED off test passed")
          (begin
            (log-error "✗ LED off test failed")
            (error "LED should be off but state is true"))))
    
    (log-info "LED basic test completed successfully")
    #t))

(define (test-led-sequence)
  "Test LED blinking sequence"
  (begin
    (log-info "Starting LED sequence test")
    
    ;; Blink LED 3 times
    (define (blink-led times)
      (if (> times 0)
          (begin
            (led-on)
            (sleep 0.001)
            (led-off)
            (sleep 0.001)
            (blink-led (- times 1)))
          #t))
    
    (blink-led 3)
    (log-info "LED sequence test completed successfully")
    #t))

(define (run-led-tests)
  "Run all LED tests"
  (begin
    (log-info "=== Running Steel LED Tests ===")
    (test-led-basic)
    (test-led-sequence)
    (log-info "=== All LED tests passed ===")
    #t))

;; Run the tests
(run-led-tests)