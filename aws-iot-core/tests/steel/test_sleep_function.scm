;; Steel test for sleep functionality
;; This test verifies sleep operations with different durations

(begin
  (log-info "=== Starting Sleep Function Test ===")
  
  ;; Test basic sleep operation
  (log-info "Testing basic sleep (0.1 seconds)...")
  (let ((start-time (current-time)))
    (sleep 0.1)
    (let ((end-time (current-time)))
      (let ((elapsed (- end-time start-time)))
        (if (>= elapsed 0.09) ; Allow some tolerance
            (log-info "✓ Basic sleep test: PASSED")
            (begin
              (log-error (format "✗ Basic sleep test: FAILED - elapsed time: ~a" elapsed))
              (error "Sleep duration was too short"))))))
  
  ;; Test zero duration sleep
  (log-info "Testing zero duration sleep...")
  (let ((start-time (current-time)))
    (sleep 0)
    (let ((end-time (current-time)))
      (let ((elapsed (- end-time start-time)))
        (if (< elapsed 0.01) ; Should be very fast
            (log-info "✓ Zero duration sleep test: PASSED")
            (begin
              (log-error (format "✗ Zero duration sleep test: FAILED - elapsed time: ~a" elapsed))
              (error "Zero duration sleep took too long"))))))
  
  ;; Test multiple short sleeps
  (log-info "Testing multiple short sleeps...")
  (let ((start-time (current-time)))
    (sleep 0.05)
    (sleep 0.05)
    (sleep 0.05)
    (let ((end-time (current-time)))
      (let ((elapsed (- end-time start-time)))
        (if (>= elapsed 0.14) ; 3 * 0.05 = 0.15, allow tolerance
            (log-info "✓ Multiple short sleeps test: PASSED")
            (begin
              (log-error (format "✗ Multiple short sleeps test: FAILED - elapsed time: ~a" elapsed))
              (error "Multiple sleeps duration was too short"))))))
  
  ;; Test sleep with LED operations
  (log-info "Testing sleep with LED operations...")
  (let ((start-time (current-time)))
    (led-on)
    (sleep 0.1)
    (led-off)
    (sleep 0.1)
    (let ((end-time (current-time)))
      (let ((elapsed (- end-time start-time)))
        (if (>= elapsed 0.18) ; 2 * 0.1 = 0.2, allow tolerance
            (log-info "✓ Sleep with LED operations test: PASSED")
            (begin
              (log-error (format "✗ Sleep with LED operations test: FAILED - elapsed time: ~a" elapsed))
              (error "Sleep with LED operations duration was too short"))))))
  
  ;; Test sleep in loop
  (log-info "Testing sleep in loop...")
  (define (sleep-loop count total-time)
    (if (> count 0)
        (begin
          (sleep 0.02)
          (sleep-loop (- count 1) (+ total-time 0.02)))
        total-time))
  
  (let ((start-time (current-time)))
    (let ((expected-time (sleep-loop 5 0)))
      (let ((end-time (current-time)))
        (let ((elapsed (- end-time start-time)))
          (if (>= elapsed 0.09) ; 5 * 0.02 = 0.1, allow tolerance
              (log-info "✓ Sleep in loop test: PASSED")
              (begin
                (log-error (format "✗ Sleep in loop test: FAILED - elapsed time: ~a" elapsed))
                (error "Sleep in loop duration was too short")))))))
  
  ;; Test sleep with device info query
  (log-info "Testing sleep with device info query...")
  (let ((start-time (current-time)))
    (let ((info1 (device-info)))
      (sleep 0.05)
      (let ((info2 (device-info)))
        (let ((end-time (current-time)))
          (let ((elapsed (- end-time start-time)))
            (if (and (>= elapsed 0.04) ; Allow tolerance
                     (equal? (get info1 'device-id) (get info2 'device-id)))
                (log-info "✓ Sleep with device info test: PASSED")
                (begin
                  (log-error (format "✗ Sleep with device info test: FAILED - elapsed: ~a" elapsed))
                  (error "Sleep with device info test failed"))))))))
  
  ;; Test nested sleep calls
  (log-info "Testing nested sleep calls...")
  (define (nested-sleep-test depth)
    (if (> depth 0)
        (begin
          (sleep 0.01)
          (nested-sleep-test (- depth 1)))
        #t))
  
  (let ((start-time (current-time)))
    (nested-sleep-test 3)
    (let ((end-time (current-time)))
      (let ((elapsed (- end-time start-time)))
        (if (>= elapsed 0.025) ; 3 * 0.01 = 0.03, allow tolerance
            (log-info "✓ Nested sleep calls test: PASSED")
            (begin
              (log-error (format "✗ Nested sleep calls test: FAILED - elapsed time: ~a" elapsed))
              (error "Nested sleep calls duration was too short"))))))
  
  (log-info "=== Sleep Function Test Completed Successfully ===")
  #t)