;; Steel test for device info functionality
;; This test verifies device information queries and system monitoring

(begin
  (log-info "=== Starting Device Info Test ===")
  
  ;; Test basic device info query
  (log-info "Testing basic device info query...")
  (let ((info (device-info)))
    (if (and info
             (get info 'device-id)
             (get info 'platform)
             (get info 'version))
        (begin
          (log-info (format "✓ Device info query: PASSED"))
          (log-info (format "  Device ID: ~a" (get info 'device-id)))
          (log-info (format "  Platform: ~a" (get info 'platform)))
          (log-info (format "  Version: ~a" (get info 'version))))
        (begin
          (log-error "✗ Device info query: FAILED - missing required fields")
          (error "Device info missing required fields"))))
  
  ;; Test device info consistency
  (log-info "Testing device info consistency...")
  (let ((info1 (device-info)))
    (let ((info2 (device-info)))
      (if (and (equal? (get info1 'device-id) (get info2 'device-id))
               (equal? (get info1 'platform) (get info2 'platform))
               (equal? (get info1 'version) (get info2 'version)))
          (log-info "✓ Device info consistency: PASSED")
          (begin
            (log-error "✗ Device info consistency: FAILED")
            (error "Device info should be consistent across calls")))))
  
  ;; Test memory info query
  (log-info "Testing memory info query...")
  (let ((memory (memory-info)))
    (if (and memory
             (get memory 'total-bytes)
             (get memory 'free-bytes)
             (get memory 'used-bytes))
        (begin
          (log-info "✓ Memory info query: PASSED")
          (log-info (format "  Total: ~a bytes" (get memory 'total-bytes)))
          (log-info (format "  Free: ~a bytes" (get memory 'free-bytes)))
          (log-info (format "  Used: ~a bytes" (get memory 'used-bytes)))
          
          ;; Verify memory calculations make sense
          (let ((total (get memory 'total-bytes))
                (free (get memory 'free-bytes))
                (used (get memory 'used-bytes)))
            (if (= total (+ free used))
                (log-info "✓ Memory calculations: PASSED")
                (begin
                  (log-error "✗ Memory calculations: FAILED - total != free + used")
                  (error "Memory calculations are inconsistent")))))
        (begin
          (log-error "✗ Memory info query: FAILED - missing required fields")
          (error "Memory info missing required fields"))))
  
  ;; Test uptime info query
  (log-info "Testing uptime info query...")
  (let ((uptime (uptime-info)))
    (if (and uptime
             (get uptime 'uptime-seconds)
             (>= (get uptime 'uptime-seconds) 0))
        (begin
          (log-info "✓ Uptime info query: PASSED")
          (log-info (format "  Uptime: ~a seconds" (get uptime 'uptime-seconds))))
        (begin
          (log-error "✗ Uptime info query: FAILED")
          (error "Uptime info invalid or missing"))))
  
  ;; Test system info over time
  (log-info "Testing system info over time...")
  (let ((uptime1 (get (uptime-info) 'uptime-seconds)))
    (sleep 0.1)
    (let ((uptime2 (get (uptime-info) 'uptime-seconds)))
      (if (>= uptime2 uptime1)
          (log-info "✓ Uptime progression: PASSED")
          (begin
            (log-error "✗ Uptime progression: FAILED - uptime went backwards")
            (error "Uptime should not decrease")))))
  
  ;; Test device info with operations
  (log-info "Testing device info during operations...")
  (let ((info-before (device-info)))
    (led-on)
    (sleep 0.05)
    (let ((info-during (device-info)))
      (led-off)
      (let ((info-after (device-info)))
        (if (and (equal? (get info-before 'device-id) (get info-during 'device-id))
                 (equal? (get info-during 'device-id) (get info-after 'device-id)))
            (log-info "✓ Device info stability during operations: PASSED")
            (begin
              (log-error "✗ Device info stability: FAILED")
              (error "Device info should remain stable during operations"))))))
  
  ;; Test multiple concurrent info queries
  (log-info "Testing multiple concurrent info queries...")
  (define (info-query-test count results)
    (if (> count 0)
        (let ((info (device-info)))
          (info-query-test (- count 1) (cons info results)))
        results))
  
  (let ((results (info-query-test 5 '())))
    (if (= (length results) 5)
        (begin
          ;; Check that all results have the same device ID
          (let ((first-id (get (car results) 'device-id)))
            (define (check-consistency lst)
              (if (null? lst)
                  #t
                  (if (equal? (get (car lst) 'device-id) first-id)
                      (check-consistency (cdr lst))
                      #f)))
            
            (if (check-consistency results)
                (log-info "✓ Multiple concurrent queries: PASSED")
                (begin
                  (log-error "✗ Multiple concurrent queries: FAILED - inconsistent results")
                  (error "Concurrent queries returned inconsistent results")))))
        (begin
          (log-error "✗ Multiple concurrent queries: FAILED - wrong number of results")
          (error "Wrong number of query results"))))
  
  ;; Test info queries with error conditions
  (log-info "Testing info queries robustness...")
  (define (robust-info-test iterations)
    (if (> iterations 0)
        (begin
          (let ((info (device-info)))
            (if (and info (get info 'device-id))
                (robust-info-test (- iterations 1))
                (error "Info query failed during robustness test"))))
        #t))
  
  (robust-info-test 10)
  (log-info "✓ Info queries robustness: PASSED")
  
  ;; Test system resource monitoring
  (log-info "Testing system resource monitoring...")
  (let ((memory-before (memory-info)))
    ;; Perform some operations that might affect memory
    (define (memory-test-operations count)
      (if (> count 0)
          (begin
            (let ((temp-info (device-info)))
              (sleep 0.01)
              (memory-test-operations (- count 1))))
          #t))
    
    (memory-test-operations 10)
    
    (let ((memory-after (memory-info)))
      ;; Memory should still be valid after operations
      (if (and (get memory-after 'total-bytes)
               (>= (get memory-after 'free-bytes) 0)
               (<= (get memory-after 'used-bytes) (get memory-after 'total-bytes)))
          (log-info "✓ System resource monitoring: PASSED")
          (begin
            (log-error "✗ System resource monitoring: FAILED")
            (error "System resources in invalid state after operations")))))
  
  (log-info "=== Device Info Test Completed Successfully ===")
  #t)