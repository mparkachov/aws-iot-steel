;; Steel example: System Monitor
;; This example demonstrates continuous system monitoring with LED indicators

(begin
  (log-info "=== System Monitor Example Starting ===")
  
  ;; Configuration
  (define monitor-interval 2.0)  ; Check every 2 seconds
  (define memory-threshold 0.75) ; Alert if memory usage > 75%
  (define max-iterations 10)     ; Run for 10 iterations in demo
  
  ;; System monitoring functions
  (define (get-system-status)
    (let ((device (device-info))
          (memory (memory-info))
          (uptime (uptime-info)))
      (list
        (cons 'device-id (get device 'device-id))
        (cons 'platform (get device 'platform))
        (cons 'memory-total (get memory 'total-bytes))
        (cons 'memory-used (get memory 'used-bytes))
        (cons 'memory-free (get memory 'free-bytes))
        (cons 'memory-usage-percent (/ (get memory 'used-bytes) (get memory 'total-bytes)))
        (cons 'uptime-seconds (get uptime 'uptime-seconds))
        (cons 'timestamp (current-time)))))
  
  (define (format-bytes bytes)
    (cond
      [(>= bytes 1048576) (format "~a MB" (/ bytes 1048576))]
      [(>= bytes 1024) (format "~a KB" (/ bytes 1024))]
      [else (format "~a bytes" bytes)]))
  
  (define (display-system-status status)
    (log-info "--- System Status ---")
    (log-info (format "Device: ~a (~a)" 
                     (get status 'device-id) 
                     (get status 'platform)))
    (log-info (format "Uptime: ~a seconds" (get status 'uptime-seconds)))
    (log-info (format "Memory: ~a / ~a (~a%)"
                     (format-bytes (get status 'memory-used))
                     (format-bytes (get status 'memory-total))
                     (round (* (get status 'memory-usage-percent) 100))))
    (log-info (format "Free Memory: ~a" (format-bytes (get status 'memory-free)))))
  
  (define (check-alerts status)
    (let ((memory-usage (get status 'memory-usage-percent)))
      (if (> memory-usage memory-threshold)
          (begin
            (log-warn (format "HIGH MEMORY USAGE ALERT: ~a%" 
                             (round (* memory-usage 100))))
            ;; Flash LED to indicate alert
            (led-on)
            (sleep 0.2)
            (led-off)
            (sleep 0.1)
            (led-on)
            (sleep 0.2)
            (led-off)
            #t)
          (begin
            ;; Brief LED pulse to show system is healthy
            (led-on)
            (sleep 0.05)
            (led-off)
            #f))))
  
  (define (log-status-to-history status alert-triggered)
    ;; In a real system, this might store to persistent storage
    (log-debug (format "Status logged: memory=~a%, alert=~a"
                      (round (* (get status 'memory-usage-percent) 100))
                      alert-triggered)))
  
  ;; Main monitoring loop
  (define (monitor-system iteration)
    (if (> iteration 0)
        (begin
          (log-info (format "=== Monitoring Cycle ~a ====" (- max-iterations iteration -1)))
          
          ;; Get current system status
          (let ((status (get-system-status)))
            
            ;; Display status information
            (display-system-status status)
            
            ;; Check for alerts
            (let ((alert-triggered (check-alerts status)))
              
              ;; Log to history
              (log-status-to-history status alert-triggered)
              
              ;; Wait before next check
              (log-debug (format "Waiting ~a seconds until next check..." monitor-interval))
              (sleep monitor-interval)
              
              ;; Continue monitoring
              (monitor-system (- iteration 1)))))
        (log-info "Monitoring cycle completed")))
  
  ;; Startup sequence
  (log-info "Initializing system monitor...")
  (log-info (format "Monitor interval: ~a seconds" monitor-interval))
  (log-info (format "Memory threshold: ~a%" (* memory-threshold 100)))
  (log-info (format "Demo iterations: ~a" max-iterations))
  
  ;; Initial system check
  (log-info "Performing initial system check...")
  (let ((initial-status (get-system-status)))
    (display-system-status initial-status)
    
    ;; LED startup sequence
    (log-info "System monitor ready - LED startup sequence")
    (led-on)
    (sleep 0.5)
    (led-off)
    (sleep 0.2)
    (led-on)
    (sleep 0.2)
    (led-off)
    
    ;; Start monitoring
    (log-info "Starting continuous monitoring...")
    (monitor-system max-iterations))
  
  ;; Shutdown sequence
  (log-info "System monitor shutting down...")
  (led-on)
  (sleep 0.1)
  (led-off)
  (sleep 0.1)
  (led-on)
  (sleep 0.1)
  (led-off)
  
  (log-info "=== System Monitor Example Completed ===")
  #t)