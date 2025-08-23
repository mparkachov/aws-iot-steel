;; Steel test for LED control functionality
;; This test verifies LED on/off operations and state queries
(begin
  (log-info "Starting LED control test")
  ;; Test LED on
  (let ((result (led-on)))
    (if result
        (log-info "LED on test: PASSED")
        (begin
          (log-error "LED on test: FAILED")
          (error "LED on returned false"))))
  ;; Test LED off
  (let ((result (led-off)))
    (if (not result)
        (log-info "LED off test: PASSED")
        (begin
          (log-error "LED off test: FAILED")
          (error "LED off returned true"))))
  ;; Test LED state query
  (let ((state (led-state)))
    (if (not state)
        (log-info "LED state test: PASSED")
        (begin
          (log-error "LED state test: FAILED")
          (error "LED state should be false after turning off"))))
  (log-info "LED control test completed successfully")
  #t)