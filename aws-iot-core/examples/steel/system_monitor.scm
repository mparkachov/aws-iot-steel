;; System Monitor Example
;; This example demonstrates system information monitoring

(define (format-bytes bytes)
  "Format bytes into human readable format"
  (cond
    ((> bytes 1048576) (string-append (->string (/ bytes 1048576)) " MB"))
    ((> bytes 1024) (string-append (->string (/ bytes 1024)) " KB"))
    (else (string-append (->string bytes) " bytes"))))

(define (format-uptime seconds)
  "Format uptime into human readable format"
  (let ((hours (floor (/ seconds 3600)))
        (minutes (floor (/ (modulo seconds 3600) 60)))
        (secs (modulo seconds 60)))
    (string-append (->string hours) "h " 
                   (->string minutes) "m " 
                   (->string secs) "s")))

(define (display-system-info)
  "Display comprehensive system information"
  (begin
    (log-info "=== System Information ===")
    
    ;; Device information
    (let ((device (device-info)))
      (log-info "Device Information:")
      (log-info (string-append "  " (->string device))))
    
    ;; Memory information
    (let ((memory (memory-info)))
      (log-info "Memory Information:")
      (log-info (string-append "  " (->string memory))))
    
    ;; Uptime information
    (let ((up (uptime)))
      (log-info "System Uptime:")
      (log-info (string-append "  " (format-uptime up))))
    
    (log-info "=== End System Information ===")))

(define (monitor-system duration interval)
  "Monitor system for specified duration with given interval"
  (begin
    (log-info (string-append "Starting system monitoring for " 
                            (->string duration) " seconds"))
    (log-info (string-append "Update interval: " (->string interval) " seconds"))
    
    (define (monitor-loop remaining)
      (if (> remaining 0)
          (begin
            (display-system-info)
            
            ;; Visual indicator - blink LED
            (led-on)
            (sleep 0.1)
            (led-off)
            
            ;; Wait for next update
            (sleep (- interval 0.1))
            (monitor-loop (- remaining interval)))
          (log-info "Monitoring completed")))
    
    (monitor-loop duration)))

(define (main)
  "Main function - monitor system for 10 seconds"
  (begin
    (log-info "=== System Monitor Example ===")
    (display-system-info)
    (log-info "")
    (monitor-system 10 2)
    (log-info "Example completed successfully")))

;; Run the example
(main)