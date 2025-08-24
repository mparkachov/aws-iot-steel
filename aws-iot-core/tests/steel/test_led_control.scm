;; Steel test for LED control functionality
;; This test verifies LED on/off operations and state queries

(begin
  (log-info "=== Starting LED Control Test ===")
  
  ;; Test LED on operation
  (log-info "Testing LED on...")
  (let ((result (led-on)))
    (if result
        (log-info "✓ LED on test: PASSED")
        (begin
          (log-error "✗ LED on test: FAILED - led-on returned false")
          (error "LED on operation failed"))))
  
  ;; Verify LED state is on
  (log-info "Verifying LED state is on...")
  (let ((state (led-state)))
    (if state
        (log-info "✓ LED state verification: PASSED - LED is on")
        (begin
          (log-error "✗ LED state verification: FAILED - LED should be on")
          (error "LED state should be on after led-on call"))))
  
  ;; Test LED off operation
  (log-info "Testing LED off...")
  (let ((result (led-off)))
    (if (not result)
        (log-info "✓ LED off test: PASSED")
        (begin
          (log-error "✗ LED off test: FAILED - led-off returned true")
          (error "LED off operation failed"))))
  
  ;; Verify LED state is off
  (log-info "Verifying LED state is off...")
  (let ((state (led-state)))
    (if (not state)
        (log-info "✓ LED state verification: PASSED - LED is off")
        (begin
          (log-error "✗ LED state verification: FAILED - LED should be off")
          (error "LED state should be off after led-off call"))))
  
  ;; Test rapid LED toggling
  (log-info "Testing rapid LED toggling...")
  (define (toggle-test count)
    (if (> count 0)
        (begin
          (if (= (modulo count 2) 0)
              (led-on)
              (led-off))
          (toggle-test (- count 1)))
        #t))
  
  (toggle-test 10)
  (log-info "✓ Rapid LED toggling: PASSED")
  
  ;; Test LED state consistency
  (log-info "Testing LED state consistency...")
  (led-on)
  (let ((state1 (led-state)))
    (let ((state2 (led-state)))
      (if (and state1 state2 (equal? state1 state2))
          (log-info "✓ LED state consistency: PASSED")
          (begin
            (log-error "✗ LED state consistency: FAILED")
            (error "LED state should be consistent across calls")))))
  
  ;; Final cleanup
  (led-off)
  
  (log-info "=== LED Control Test Completed Successfully ===")
  #t)