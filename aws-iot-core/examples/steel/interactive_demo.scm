;; Steel example: Interactive Demo
;; This example demonstrates various Steel capabilities in an interactive format

(begin
  (log-info "=== Interactive Steel Demo Starting ===")
  
  ;; Demo configuration
  (define demo-steps 8)
  (define step-delay 1.5)
  
  ;; Utility functions
  (define (demo-step step-num title)
    (log-info "")
    (log-info (format "=== Step ~a/~a: ~a ===" step-num demo-steps title))
    (sleep 0.5))
  
  (define (wait-for-step)
    (log-info (format "Waiting ~a seconds before next step..." step-delay))
    (sleep step-delay))
  
  ;; Step 1: Introduction and Device Info
  (demo-step 1 "Device Information")
  (log-info "Welcome to the Steel Interactive Demo!")
  (log-info "Let's start by examining our device...")
  
  (let ((device (device-info)))
    (log-info (format "Device ID: ~a" (get device 'device-id)))
    (log-info (format "Platform: ~a" (get device 'platform)))
    (log-info (format "Version: ~a" (get device 'version)))
    (log-info (format "Firmware: ~a" (get device 'firmware-version))))
  
  (wait-for-step)
  
  ;; Step 2: LED Control Demonstration
  (demo-step 2 "LED Control")
  (log-info "Now let's control the LED...")
  
  (log-info "Turning LED ON")
  (led-on)
  (log-info (format "LED state: ~a" (if (led-state) "ON" "OFF")))
  (sleep 1.0)
  
  (log-info "Turning LED OFF")
  (led-off)
  (log-info (format "LED state: ~a" (if (led-state) "ON" "OFF")))
  
  (wait-for-step)
  
  ;; Step 3: LED Patterns
  (demo-step 3 "LED Patterns")
  (log-info "Creating LED patterns...")
  
  (define (blink-pattern count delay)
    (if (> count 0)
        (begin
          (log-debug (format "Blink ~a" (- 4 count)))
          (led-on)
          (sleep delay)
          (led-off)
          (sleep delay)
          (blink-pattern (- count 1) delay))
        #t))
  
  (log-info "Pattern 1: Slow blinks")
  (blink-pattern 3 0.3)
  
  (log-info "Pattern 2: Fast blinks")
  (blink-pattern 5 0.1)
  
  (wait-for-step)
  
  ;; Step 4: Memory and System Information
  (demo-step 4 "System Information")
  (log-info "Examining system resources...")
  
  (let ((memory (memory-info))
        (uptime (uptime-info)))
    (log-info (format "Memory Total: ~a bytes" (get memory 'total-bytes)))
    (log-info (format "Memory Used: ~a bytes" (get memory 'used-bytes)))
    (log-info (format "Memory Free: ~a bytes" (get memory 'free-bytes)))
    (log-info (format "Memory Usage: ~a%" 
                     (round (* (/ (get memory 'used-bytes) (get memory 'total-bytes)) 100))))
    (log-info (format "System Uptime: ~a seconds" (get uptime 'uptime-seconds))))
  
  (wait-for-step)
  
  ;; Step 5: Mathematical Calculations
  (demo-step 5 "Mathematical Operations")
  (log-info "Demonstrating Steel's mathematical capabilities...")
  
  (define (fibonacci n)
    (if (<= n 1)
        n
        (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))
  
  (define (factorial n)
    (if (<= n 1)
        1
        (* n (factorial (- n 1)))))
  
  (log-info "Calculating Fibonacci sequence:")
  (define (show-fibonacci count)
    (if (> count 0)
        (begin
          (log-info (format "  fib(~a) = ~a" (- 8 count) (fibonacci (- 8 count))))
          (show-fibonacci (- count 1)))
        #t))
  (show-fibonacci 8)
  
  (log-info "Calculating factorials:")
  (log-info (format "  5! = ~a" (factorial 5)))
  (log-info (format "  7! = ~a" (factorial 7)))
  
  (wait-for-step)
  
  ;; Step 6: Data Processing
  (demo-step 6 "Data Processing")
  (log-info "Processing sensor data simulation...")
  
  (define (generate-sensor-data count)
    (define (generate-reading index)
      (list
        (cons 'sensor-id (format "sensor-~a" index))
        (cons 'temperature (+ 20 (* 5 (sin (* index 0.5)))))
        (cons 'humidity (+ 50 (* 10 (cos (* index 0.3)))))
        (cons 'timestamp (+ (current-time) index))))
    
    (define (collect-readings remaining readings)
      (if (> remaining 0)
          (collect-readings (- remaining 1) 
                           (cons (generate-reading (- count remaining)) readings))
          readings))
    
    (collect-readings count '()))
  
  (define (analyze-sensor-data readings)
    (define (sum-temperatures readings total count)
      (if (null? readings)
          (/ total count)
          (sum-temperatures (cdr readings) 
                           (+ total (get (car readings) 'temperature))
                           (+ count 1))))
    
    (let ((avg-temp (sum-temperatures readings 0 0)))
      (log-info (format "Analyzed ~a sensor readings" (length readings)))
      (log-info (format "Average temperature: ~a°C" (round avg-temp)))
      avg-temp))
  
  (let ((sensor-data (generate-sensor-data 10)))
    (log-info "Generated sensor data:")
    (define (display-readings readings count)
      (if (and (not (null? readings)) (> count 0))
          (let ((reading (car readings)))
            (log-info (format "  ~a: temp=~a°C, humidity=~a%"
                             (get reading 'sensor-id)
                             (round (get reading 'temperature))
                             (round (get reading 'humidity))))
            (display-readings (cdr readings) (- count 1)))
          #t))
    
    (display-readings sensor-data 5)  ; Show first 5 readings
    (if (> (length sensor-data) 5)
        (log-info (format "  ... and ~a more readings" (- (length sensor-data) 5))))
    
    (analyze-sensor-data sensor-data))
  
  (wait-for-step)
  
  ;; Step 7: Control Flow and Conditionals
  (demo-step 7 "Control Flow")
  (log-info "Demonstrating control flow and decision making...")
  
  (define (system-status-check)
    (let ((memory (memory-info))
          (device (device-info)))
      (let ((memory-usage (/ (get memory 'used-bytes) (get memory 'total-bytes))))
        
        (log-info "Performing system status evaluation...")
        
        (cond
          [(> memory-usage 0.9)
           (begin
             (log-warn "CRITICAL: Memory usage very high!")
             (led-on)
             (sleep 0.5)
             (led-off)
             "CRITICAL")]
          [(> memory-usage 0.7)
           (begin
             (log-warn "WARNING: Memory usage high")
             (led-on)
             (sleep 0.2)
             (led-off)
             "WARNING")]
          [(> memory-usage 0.5)
           (begin
             (log-info "CAUTION: Memory usage moderate")
             "CAUTION")]
          [else
           (begin
             (log-info "OK: Memory usage normal")
             "OK")])
        
        (log-info (format "System status: ~a (Memory: ~a%)"
                         (cond
                           [(> memory-usage 0.9) "CRITICAL"]
                           [(> memory-usage 0.7) "WARNING"]
                           [(> memory-usage 0.5) "CAUTION"]
                           [else "OK"])
                         (round (* memory-usage 100)))))))
  
  (system-status-check)
  
  ;; Demonstrate loops
  (log-info "Demonstrating countdown loop:")
  (define (countdown n)
    (if (> n 0)
        (begin
          (log-info (format "  Countdown: ~a" n))
          (led-on)
          (sleep 0.1)
          (led-off)
          (sleep 0.2)
          (countdown (- n 1)))
        (log-info "  Countdown complete!")))
  
  (countdown 5)
  
  (wait-for-step)
  
  ;; Step 8: Final Demonstration
  (demo-step 8 "Grand Finale")
  (log-info "Final demonstration combining all features...")
  
  (define (grand-finale)
    (log-info "Performing comprehensive system demonstration...")
    
    ;; Get initial system state
    (let ((device (device-info))
          (memory (memory-info)))
      
      (log-info (format "Starting finale on device: ~a" (get device 'device-id)))
      
      ;; LED celebration sequence
      (log-info "LED celebration sequence:")
      (define (celebration-pattern)
        ;; Fast blinks
        (define (fast-blinks count)
          (if (> count 0)
              (begin
                (led-on)
                (sleep 0.05)
                (led-off)
                (sleep 0.05)
                (fast-blinks (- count 1)))
              #t))
        
        ;; Slow pulse
        (define (slow-pulse count)
          (if (> count 0)
              (begin
                (led-on)
                (sleep 0.3)
                (led-off)
                (sleep 0.3)
                (slow-pulse (- count 1)))
              #t))
        
        (fast-blinks 8)
        (sleep 0.5)
        (slow-pulse 3))
      
      (celebration-pattern)
      
      ;; Final system report
      (log-info "Final system report:")
      (let ((final-memory (memory-info))
            (final-uptime (uptime-info)))
        (log-info (format "  Device: ~a" (get device 'device-id)))
        (log-info (format "  Platform: ~a" (get device 'platform)))
        (log-info (format "  Demo duration: ~a seconds" 
                         (- (get final-uptime 'uptime-seconds) 
                            (get (uptime-info) 'uptime-seconds))))
        (log-info (format "  Memory usage: ~a%" 
                         (round (* (/ (get final-memory 'used-bytes) 
                                     (get final-memory 'total-bytes)) 100))))
        
        ;; Success indicator
        (log-info "Demo completed successfully!")
        (led-on)
        (sleep 1.0)
        (led-off))))
  
  (grand-finale)
  
  ;; Demo conclusion
  (log-info "")
  (log-info "=== Interactive Steel Demo Completed ===")
  (log-info "Thank you for exploring Steel's capabilities!")
  (log-info "This demo showcased:")
  (log-info "  • Device information queries")
  (log-info "  • LED control and patterns")
  (log-info "  • System monitoring")
  (log-info "  • Mathematical operations")
  (log-info "  • Data processing")
  (log-info "  • Control flow and conditionals")
  (log-info "  • Complex program logic")
  
  (log-info "Steel provides a powerful scripting environment for IoT devices!")
  
  #t)