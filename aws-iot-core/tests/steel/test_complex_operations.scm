;; Steel test for complex operations combining multiple features
;; This test verifies integration between different system components

(begin
  (log-info "=== Starting Complex Operations Test ===")
  
  ;; Test 1: Sensor monitoring simulation
  (log-info "Test 1: Sensor monitoring simulation")
  
  (define (simulate-sensor-reading)
    ;; Simulate reading sensor data by using device info and memory info
    (let ((device (device-info))
          (memory (memory-info)))
      (list
        (cons 'device-id (get device 'device-id))
        (cons 'memory-usage (/ (get memory 'used-bytes) (get memory 'total-bytes)))
        (cons 'timestamp (current-time)))))
  
  (define (monitor-sensors duration-seconds readings)
    (if (> duration-seconds 0)
        (begin
          (log-debug (format "Taking sensor reading... (~a seconds remaining)" duration-seconds))
          (let ((reading (simulate-sensor-reading)))
            (log-info (format "Sensor reading: device=~a, memory-usage=~a"
                             (get reading 'device-id)
                             (get reading 'memory-usage)))
            
            ;; Alert if memory usage is high (simulated threshold)
            (if (> (get reading 'memory-usage) 0.8)
                (begin
                  (log-warn "High memory usage detected!")
                  (led-on)
                  (sleep 0.1)
                  (led-off))
                (log-debug "Memory usage normal"))
            
            (sleep 0.2)
            (monitor-sensors (- duration-seconds 0.2) (cons reading readings))))
        readings))
  
  (let ((readings (monitor-sensors 1.0 '())))
    (log-info (format "Collected ~a sensor readings" (length readings)))
    (if (>= (length readings) 4)  ; Should have ~5 readings in 1 second
        (log-info "✓ Sensor monitoring simulation: PASSED")
        (begin
          (log-error "✗ Sensor monitoring simulation: FAILED - insufficient readings")
          (error "Sensor monitoring did not collect enough readings"))))
  
  ;; Test 2: System health check routine
  (log-info "Test 2: System health check routine")
  
  (define (check-system-health)
    (log-info "Starting system health check...")
    
    ;; Check device info availability
    (let ((device-check
           (let ((info (device-info)))
             (and info
                  (get info 'device-id)
                  (get info 'platform)
                  (get info 'version)))))
      
      ;; Check memory status
      (let ((memory-check
             (let ((memory (memory-info)))
               (and memory
                    (> (get memory 'total-bytes) 0)
                    (>= (get memory 'free-bytes) 0)
                    (<= (get memory 'used-bytes) (get memory 'total-bytes))))))
        
        ;; Check LED functionality
        (let ((led-check
               (begin
                 (led-on)
                 (let ((state1 (led-state)))
                   (led-off)
                   (let ((state2 (led-state)))
                     (and state1 (not state2)))))))
          
          ;; Check sleep functionality
          (let ((sleep-check
                 (let ((start (current-time)))
                   (sleep 0.1)
                   (let ((end (current-time)))
                     (>= (- end start) 0.09)))))
            
            (log-info (format "Health check results:"))
            (log-info (format "  Device info: ~a" (if device-check "✓" "✗")))
            (log-info (format "  Memory info: ~a" (if memory-check "✓" "✗")))
            (log-info (format "  LED control: ~a" (if led-check "✓" "✗")))
            (log-info (format "  Sleep function: ~a" (if sleep-check "✓" "✗")))
            
            (and device-check memory-check led-check sleep-check))))))
  
  (if (check-system-health)
      (log-info "✓ System health check: PASSED")
      (begin
        (log-error "✗ System health check: FAILED")
        (error "System health check failed")))
  
  ;; Test 3: Automated LED pattern sequence
  (log-info "Test 3: Automated LED pattern sequence")
  
  (define (led-pattern-blink count delay)
    (if (> count 0)
        (begin
          (led-on)
          (sleep delay)
          (led-off)
          (sleep delay)
          (led-pattern-blink (- count 1) delay))
        #t))
  
  (define (led-pattern-pulse count)
    (if (> count 0)
        (begin
          (led-on)
          (sleep 0.05)
          (led-off)
          (sleep 0.02)
          (led-pattern-pulse (- count 1)))
        #t))
  
  (define (led-pattern-sequence)
    (log-info "Starting LED pattern sequence...")
    
    ;; Pattern 1: Slow blinks
    (log-debug "Pattern 1: Slow blinks")
    (led-pattern-blink 3 0.1)
    
    ;; Pattern 2: Fast pulses
    (log-debug "Pattern 2: Fast pulses")
    (led-pattern-pulse 5)
    
    ;; Pattern 3: Custom sequence
    (log-debug "Pattern 3: Custom sequence")
    (led-on)
    (sleep 0.2)
    (led-off)
    (sleep 0.05)
    (led-on)
    (sleep 0.05)
    (led-off)
    (sleep 0.05)
    (led-on)
    (sleep 0.05)
    (led-off)
    
    (log-info "LED pattern sequence completed"))
  
  (led-pattern-sequence)
  (log-info "✓ LED pattern sequence: PASSED")
  
  ;; Test 4: Data processing and analysis
  (log-info "Test 4: Data processing and analysis")
  
  (define (collect-system-metrics iterations)
    (define (collect-iteration count metrics)
      (if (> count 0)
          (let ((memory (memory-info))
                (uptime (uptime-info))
                (timestamp (current-time)))
            (let ((metric (list
                          (cons 'iteration (- iterations count -1))
                          (cons 'memory-free (get memory 'free-bytes))
                          (cons 'memory-used (get memory 'used-bytes))
                          (cons 'uptime (get uptime 'uptime-seconds))
                          (cons 'timestamp timestamp))))
              (sleep 0.05)  ; Small delay between measurements
              (collect-iteration (- count 1) (cons metric metrics))))
          metrics))
    (collect-iteration iterations '()))
  
  (define (analyze-metrics metrics)
    (log-info "Analyzing collected metrics...")
    
    ;; Calculate average memory usage
    (define (sum-memory-used metrics total)
      (if (null? metrics)
          total
          (sum-memory-used (cdr metrics) 
                          (+ total (get (car metrics) 'memory-used)))))
    
    (let ((total-memory-used (sum-memory-used metrics 0))
          (count (length metrics)))
      (let ((avg-memory-used (/ total-memory-used count)))
        (log-info (format "Metrics analysis:"))
        (log-info (format "  Total samples: ~a" count))
        (log-info (format "  Average memory used: ~a bytes" avg-memory-used))
        
        ;; Verify metrics make sense
        (and (> count 0)
             (> avg-memory-used 0)
             (< avg-memory-used 10000000))))) ; Reasonable upper bound
  
  (let ((metrics (collect-system-metrics 10)))
    (if (and (= (length metrics) 10)
             (analyze-metrics metrics))
        (log-info "✓ Data processing and analysis: PASSED")
        (begin
          (log-error "✗ Data processing and analysis: FAILED")
          (error "Data processing and analysis failed"))))
  
  ;; Test 5: Error handling and recovery
  (log-info "Test 5: Error handling and recovery")
  
  (define (test-error-recovery)
    (log-info "Testing error handling and recovery...")
    
    ;; Test recovery from simulated errors
    (define (simulate-operation-with-recovery operation-name should-fail)
      (log-debug (format "Attempting operation: ~a" operation-name))
      
      (if should-fail
          (begin
            (log-warn (format "Operation ~a encountered simulated error" operation-name))
            ;; Simulate recovery actions
            (led-on)
            (sleep 0.05)
            (led-off)
            (log-info (format "Recovery actions completed for ~a" operation-name))
            #f)  ; Return failure
          (begin
            (log-debug (format "Operation ~a completed successfully" operation-name))
            #t))) ; Return success
    
    ;; Test various operations with different failure scenarios
    (let ((op1 (simulate-operation-with-recovery "sensor-read" #f))
          (op2 (simulate-operation-with-recovery "network-send" #t))
          (op3 (simulate-operation-with-recovery "data-process" #f))
          (op4 (simulate-operation-with-recovery "system-check" #t)))
      
      (log-info (format "Operation results: ~a successful, ~a failed"
                       (+ (if op1 1 0) (if op3 1 0))
                       (+ (if op2 1 0) (if op4 1 0))))
      
      ;; System should continue functioning despite some failures
      (let ((final-check (device-info)))
        (and final-check (get final-check 'device-id)))))
  
  (if (test-error-recovery)
      (log-info "✓ Error handling and recovery: PASSED")
      (begin
        (log-error "✗ Error handling and recovery: FAILED")
        (error "Error handling and recovery test failed")))
  
  ;; Test 6: Performance under load
  (log-info "Test 6: Performance under load")
  
  (define (performance-test-operations count)
    (define (operation-batch iteration)
      (if (> iteration 0)
          (begin
            ;; Perform a batch of operations
            (let ((info (device-info)))
              (led-on)
              (let ((memory (memory-info)))
                (sleep 0.01)
                (led-off)
                (log-debug (format "Batch ~a: device=~a, memory-free=~a"
                                  iteration
                                  (get info 'device-id)
                                  (get memory 'free-bytes)))))
            (operation-batch (- iteration 1)))
          #t))
    
    (let ((start-time (current-time)))
      (operation-batch count)
      (let ((end-time (current-time)))
        (let ((elapsed (- end-time start-time)))
          (log-info (format "Completed ~a operation batches in ~a seconds" count elapsed))
          (log-info (format "Average time per batch: ~a seconds" (/ elapsed count)))
          
          ;; Performance should be reasonable
          (< elapsed 2.0))))) ; Should complete 20 batches in under 2 seconds
  
  (if (performance-test-operations 20)
      (log-info "✓ Performance under load: PASSED")
      (begin
        (log-error "✗ Performance under load: FAILED")
        (error "Performance under load test failed")))
  
  ;; Test 7: State consistency verification
  (log-info "Test 7: State consistency verification")
  
  (define (verify-state-consistency)
    (log-info "Verifying system state consistency...")
    
    ;; Perform operations and verify state remains consistent
    (let ((initial-device (device-info)))
      (led-on)
      (sleep 0.1)
      (let ((during-device (device-info)))
        (led-off)
        (let ((final-device (device-info)))
          
          ;; Device info should remain consistent
          (let ((device-consistent
                 (and (equal? (get initial-device 'device-id) (get during-device 'device-id))
                      (equal? (get during-device 'device-id) (get final-device 'device-id))
                      (equal? (get initial-device 'platform) (get final-device 'platform)))))
            
            ;; LED state should be off at the end
            (let ((led-state-correct (not (led-state))))
              
              ;; Memory info should be valid
              (let ((memory (memory-info)))
                (let ((memory-valid
                       (and (> (get memory 'total-bytes) 0)
                            (>= (get memory 'free-bytes) 0)
                            (<= (get memory 'used-bytes) (get memory 'total-bytes)))))
                  
                  (log-info (format "State consistency check:"))
                  (log-info (format "  Device info consistent: ~a" (if device-consistent "✓" "✗")))
                  (log-info (format "  LED state correct: ~a" (if led-state-correct "✓" "✗")))
                  (log-info (format "  Memory info valid: ~a" (if memory-valid "✓" "✗")))
                  
                  (and device-consistent led-state-correct memory-valid))))))))
  
  (if (verify-state-consistency)
      (log-info "✓ State consistency verification: PASSED")
      (begin
        (log-error "✗ State consistency verification: FAILED")
        (error "State consistency verification failed")))
  
  ;; Final system status check
  (log-info "Performing final system status check...")
  (let ((final-device (device-info))
        (final-memory (memory-info))
        (final-uptime (uptime-info)))
    
    (log-info (format "Final system status:"))
    (log-info (format "  Device: ~a (~a)" 
                     (get final-device 'device-id) 
                     (get final-device 'platform)))
    (log-info (format "  Memory: ~a/~a bytes used" 
                     (get final-memory 'used-bytes)
                     (get final-memory 'total-bytes)))
    (log-info (format "  Uptime: ~a seconds" (get final-uptime 'uptime-seconds)))
    (log-info (format "  LED state: ~a" (if (led-state) "ON" "OFF"))))
  
  ;; Ensure LED is off at the end
  (led-off)
  
  (log-info "=== Complex Operations Test Completed Successfully ===")
  #t)