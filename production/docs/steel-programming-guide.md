# Steel Programming Guide
## AWS IoT Steel System

### Document Information
- **Version**: 1.0.0
- **Last Updated**: 2024-01-01
- **Audience**: Developers, IoT Engineers
- **Prerequisites**: Basic Scheme/Lisp knowledge, IoT concepts

---

## Introduction

Steel is a Scheme-based programming language that runs on the AWS IoT Steel System. It provides a high-level, dynamic programming environment for IoT devices while maintaining the performance and reliability of the underlying Rust runtime.

### Key Features
- **Dynamic Programming**: Load and execute programs remotely via AWS IoT
- **Scheme Language**: Full-featured Scheme with modern extensions
- **Hardware Integration**: Direct access to device hardware through Rust APIs
- **Cloud Connectivity**: Built-in AWS IoT Core integration
- **Real-time Execution**: Low-latency program execution and updates

---

## Steel Language Basics

### Syntax Overview

Steel follows standard Scheme syntax with some modern extensions:

```scheme
;; Comments start with semicolon
;; This is a single-line comment

#|
This is a
multi-line comment
|#

;; Basic data types
42                    ; Integer
3.14159              ; Float
"Hello, World!"      ; String
#t                   ; Boolean true
#f                   ; Boolean false
'symbol              ; Symbol
'(1 2 3)            ; List
#(1 2 3)            ; Vector
```

### Variables and Functions

```scheme
;; Define variables
(define pi 3.14159)
(define greeting "Hello, IoT!")

;; Define functions
(define (square x)
  (* x x))

;; Lambda expressions
(define add-one (lambda (x) (+ x 1)))

;; Local variables
(let ((x 10)
      (y 20))
  (+ x y))  ; Returns 30
```

### Control Flow

```scheme
;; Conditionals
(if (> temperature 25)
    (led-on)
    (led-off))

;; Multiple conditions
(cond
  [(< temperature 0) "freezing"]
  [(< temperature 20) "cold"]
  [(< temperature 30) "warm"]
  [else "hot"])

;; Loops
(define (countdown n)
  (if (> n 0)
      (begin
        (log "info" (format "Count: ~a" n))
        (sleep 1)
        (countdown (- n 1)))
      (log "info" "Done!")))
```

---

## Hardware API Reference

### LED Control

```scheme
;; Turn LED on
(led-on)

;; Turn LED off
(led-off)

;; Check LED state
(led-state)  ; Returns #t if on, #f if off

;; Toggle LED
(define (toggle-led)
  (if (led-state)
      (led-off)
      (led-on)))
```

### Sleep and Timing

```scheme
;; Sleep for specified seconds (can be fractional)
(sleep 1.5)    ; Sleep for 1.5 seconds
(sleep 0.1)    ; Sleep for 100 milliseconds

;; Get current timestamp
(current-time)  ; Returns Unix timestamp

;; Measure execution time
(define start-time (current-time))
(some-operation)
(define elapsed (- (current-time) start-time))
(log "info" (format "Operation took ~a seconds" elapsed))
```

### Sensor Data

```scheme
;; Read all sensor data
(define sensors (read-sensors))

;; Access specific sensor values
(define temp (get sensors 'temperature))
(define humidity (get sensors 'humidity))
(define battery (get sensors 'battery))

;; Example sensor data structure:
;; '((temperature . 23.5)
;;   (humidity . 65.2)
;;   (battery . 87)
;;   (timestamp . 1640995200))
```

### System Information

```scheme
;; Get device information
(define device-info (device-info))

;; Access device properties
(define device-id (get device-info 'device_id))
(define platform (get device-info 'platform))
(define version (get device-info 'version))

;; Get system uptime
(uptime)  ; Returns uptime in seconds

;; Get memory information
(define memory (memory-info))
(define free-memory (get memory 'free))
(define total-memory (get memory 'total))
```

---

## Cloud Integration API

### MQTT Messaging

```scheme
;; Publish a message
(mqtt-publish "sensors/temperature" "23.5")

;; Publish JSON data
(mqtt-publish "sensors/data" 
              (json-encode '((temperature . 23.5)
                           (humidity . 65.2)
                           (timestamp . 1640995200))))

;; Subscribe to a topic (in event handler)
(register-event-handler "mqtt-message"
  (lambda (topic payload)
    (log "info" (format "Received: ~a on ~a" payload topic))))
```

### Device Shadow

```scheme
;; Update device shadow
(shadow-update "temperature" 23.5)
(shadow-update "status" "running")

;; Update multiple shadow properties
(shadow-update "sensor-data" 
               (json-encode '((temperature . 23.5)
                            (humidity . 65.2)
                            (battery . 87))))

;; Shadow updates are automatically synchronized with AWS IoT Core
```

### Logging

```scheme
;; Log levels: "debug", "info", "warn", "error"
(log "info" "System started")
(log "warn" "Temperature high")
(log "error" "Sensor failure")

;; Formatted logging
(log "info" (format "Temperature: ~a°C" temperature))

;; Structured logging with JSON
(log "info" (json-encode '((event . "sensor-reading")
                          (temperature . 23.5)
                          (timestamp . 1640995200))))
```

---

## Data Storage API

```scheme
;; Store data persistently
(store "last-temperature" "23.5")
(store "device-config" (json-encode config-data))

;; Load stored data
(define last-temp (load "last-temperature"))
(define config (if last-temp 
                   (json-decode (load "device-config"))
                   default-config))

;; Delete stored data
(delete "old-data-key")

;; Check if key exists
(if (load "sensor-calibration")
    (log "info" "Using stored calibration")
    (log "warn" "No calibration data found"))
```

---

## Timer and Scheduling API

```scheme
;; Set a one-time timer
(set-timer "sensor-reading" 30.0
  (lambda ()
    (let ((sensors (read-sensors)))
      (mqtt-publish "sensors/data" (json-encode sensors)))))

;; Cancel a timer
(cancel-timer "sensor-reading")

;; Schedule recurring tasks (cron-style)
(schedule "0 */5 * * *"  ; Every 5 minutes
  (lambda ()
    (log "info" "Periodic health check")
    (shadow-update "last-heartbeat" (current-time))))

;; Schedule with simpler intervals
(schedule-interval 60  ; Every 60 seconds
  (lambda ()
    (let ((memory (memory-info)))
      (when (< (get memory 'free) 1000)
        (log "warn" "Low memory warning")))))
```

---

## Advanced Programming Patterns

### Event-Driven Programming

```scheme
;; Register event handlers
(register-event-handler "mqtt-message"
  (lambda (topic payload)
    (cond
      [(string-contains? topic "commands/led")
       (if (string=? payload "on")
           (led-on)
           (led-off))]
      [(string-contains? topic "commands/sleep")
       (sleep (string->number payload))]
      [else
       (log "warn" (format "Unknown command topic: ~a" topic))])))

;; Emit custom events
(emit-event "sensor-threshold-exceeded" 
            '((sensor . "temperature")
              (value . 35.2)
              (threshold . 30.0)))

;; Handle custom events
(register-event-handler "sensor-threshold-exceeded"
  (lambda (data)
    (led-on)
    (mqtt-publish "alerts/temperature" (json-encode data))))
```

### State Machines

```scheme
;; Define states
(define current-state 'idle)

;; State transition function
(define (transition-to new-state)
  (log "info" (format "State: ~a -> ~a" current-state new-state))
  (set! current-state new-state))

;; State machine logic
(define (handle-sensor-data sensors)
  (let ((temp (get sensors 'temperature)))
    (case current-state
      ['idle
       (when (> temp 30)
         (transition-to 'monitoring)
         (led-on))]
      ['monitoring
       (cond
         [(> temp 35) (transition-to 'alert)]
         [(< temp 25) (transition-to 'idle) (led-off)])]
      ['alert
       (mqtt-publish "alerts/temperature" (json-encode sensors))
       (when (< temp 30)
         (transition-to 'monitoring))])))
```

### Error Handling

```scheme
;; Try-catch equivalent
(define (safe-sensor-read)
  (guard (ex
          [(sensor-error? ex)
           (log "error" "Sensor read failed")
           '((temperature . 0) (error . #t))]
          [else
           (log "error" (format "Unexpected error: ~a" ex))
           '((error . #t))])
    (read-sensors)))

;; Retry logic
(define (retry-operation operation max-attempts)
  (define (attempt n)
    (guard (ex
            [else
             (if (< n max-attempts)
                 (begin
                   (log "warn" (format "Attempt ~a failed, retrying..." n))
                   (sleep 1)
                   (attempt (+ n 1)))
                 (begin
                   (log "error" "All retry attempts failed")
                   (raise ex)))])
      (operation)))
  (attempt 1))
```

---

## Example Programs

### Simple Blink Program

```scheme
;; blink_led.scm - Simple LED blinking program
(define (blink-led times interval)
  (if (> times 0)
      (begin
        (led-on)
        (sleep interval)
        (led-off)
        (sleep interval)
        (blink-led (- times 1) interval))
      (log "info" "Blinking complete")))

;; Start blinking
(log "info" "Starting LED blink program")
(blink-led 10 0.5)  ; Blink 10 times with 0.5s interval
```

### Sensor Monitoring System

```scheme
;; sensor_monitor.scm - Comprehensive sensor monitoring
(define monitoring-active #t)
(define alert-threshold 30.0)
(define check-interval 10)

(define (sensor-monitoring-loop)
  (when monitoring-active
    (let ((sensors (read-sensors)))
      (let ((temp (get sensors 'temperature))
            (humidity (get sensors 'humidity))
            (battery (get sensors 'battery)))
        
        ;; Log sensor readings
        (log "info" (format "Sensors - Temp: ~a°C, Humidity: ~a%, Battery: ~a%"
                           temp humidity battery))
        
        ;; Check temperature threshold
        (when (> temp alert-threshold)
          (log "warn" (format "Temperature alert: ~a°C > ~a°C" temp alert-threshold))
          (led-on)
          (mqtt-publish "alerts/temperature" 
                       (json-encode '((temperature . temp)
                                    (threshold . alert-threshold)
                                    (timestamp . (current-time))))))
        
        ;; Check battery level
        (when (< battery 20)
          (log "warn" (format "Low battery: ~a%" battery))
          (mqtt-publish "alerts/battery" 
                       (json-encode '((battery . battery)
                                    (timestamp . (current-time))))))
        
        ;; Update device shadow
        (shadow-update "sensor-data" (json-encode sensors))
        
        ;; Turn off LED if temperature is normal
        (when (<= temp alert-threshold)
          (led-off))
        
        ;; Schedule next check
        (set-timer "sensor-check" check-interval sensor-monitoring-loop)))))

;; Handle remote commands
(register-event-handler "mqtt-message"
  (lambda (topic payload)
    (cond
      [(string=? topic "commands/stop-monitoring")
       (set! monitoring-active #f)
       (log "info" "Monitoring stopped by remote command")]
      [(string=? topic "commands/set-threshold")
       (set! alert-threshold (string->number payload))
       (log "info" (format "Alert threshold set to ~a°C" alert-threshold))]
      [(string=? topic "commands/set-interval")
       (set! check-interval (string->number payload))
       (log "info" (format "Check interval set to ~a seconds" check-interval))])))

;; Start monitoring
(log "info" "Starting sensor monitoring system")
(sensor-monitoring-loop)
```

### Interactive Device Controller

```scheme
;; device_controller.scm - Interactive device control system
(define device-state '((led . #f)
                      (monitoring . #f)
                      (last-command . "none")
                      (uptime-start . (current-time))))

(define (update-state key value)
  (set! device-state (assoc-set device-state key value))
  (shadow-update "device-state" (json-encode device-state)))

(define (get-state key)
  (cdr (assoc key device-state)))

(define (handle-command command args)
  (update-state 'last-command command)
  (case (string->symbol command)
    ['led-on
     (led-on)
     (update-state 'led #t)
     (log "info" "LED turned on")]
    ['led-off
     (led-off)
     (update-state 'led #f)
     (log "info" "LED turned off")]
    ['led-toggle
     (if (get-state 'led)
         (handle-command "led-off" '())
         (handle-command "led-on" '()))]
    ['sleep
     (let ((duration (if (null? args) 1 (string->number (car args)))))
       (log "info" (format "Sleeping for ~a seconds" duration))
       (sleep duration))]
    ['status
     (let ((uptime (- (current-time) (get-state 'uptime-start)))
           (memory (memory-info))
           (sensors (read-sensors)))
       (mqtt-publish "status/report"
                    (json-encode '((uptime . uptime)
                                 (memory . memory)
                                 (sensors . sensors)
                                 (state . device-state)))))]
    ['start-monitoring
     (update-state 'monitoring #t)
     (start-sensor-monitoring)]
    ['stop-monitoring
     (update-state 'monitoring #f)
     (log "info" "Monitoring stopped")]
    [else
     (log "warn" (format "Unknown command: ~a" command))]))

(define (start-sensor-monitoring)
  (when (get-state 'monitoring)
    (let ((sensors (read-sensors)))
      (mqtt-publish "sensors/data" (json-encode sensors))
      (set-timer "monitoring" 30 start-sensor-monitoring))))

;; Command handler
(register-event-handler "mqtt-message"
  (lambda (topic payload)
    (when (string-contains? topic "commands/")
      (let* ((command-part (substring topic (string-length "commands/")))
             (parts (string-split payload #\space))
             (command (if (null? parts) command-part (car parts)))
             (args (if (null? parts) '() (cdr parts))))
        (handle-command command args)))))

;; Periodic status updates
(schedule "0 */10 * * *"  ; Every 10 minutes
  (lambda ()
    (handle-command "status" '())))

;; Initialize
(log "info" "Device controller initialized")
(update-state 'last-command "init")
(mqtt-publish "device/online" (json-encode '((device-id . (get (device-info) 'device_id))
                                           (timestamp . (current-time)))))
```

---

## Best Practices

### Performance Optimization

```scheme
;; Use local variables for frequently accessed data
(let ((sensors (read-sensors)))
  (let ((temp (get sensors 'temperature))
        (humidity (get sensors 'humidity)))
    ;; Process temp and humidity multiple times
    (process-temperature temp)
    (process-humidity humidity)
    (log-readings temp humidity)))

;; Batch operations when possible
(define readings '())
(define (collect-reading)
  (set! readings (cons (read-sensors) readings))
  (when (>= (length readings) 10)
    (mqtt-publish "sensors/batch" (json-encode readings))
    (set! readings '())))

;; Use appropriate data structures
(define sensor-cache (make-hash-table))
(hash-set! sensor-cache 'temperature 23.5)
(hash-ref sensor-cache 'temperature)
```

### Error Handling

```scheme
;; Always handle potential errors
(define (safe-mqtt-publish topic payload)
  (guard (ex
          [(network-error? ex)
           (log "warn" "Network error, queuing message")
           (queue-message topic payload)]
          [else
           (log "error" (format "Unexpected error: ~a" ex))])
    (mqtt-publish topic payload)))

;; Validate inputs
(define (set-led-brightness level)
  (cond
    [(not (number? level))
     (log "error" "Brightness level must be a number")]
    [(or (< level 0) (> level 100))
     (log "error" "Brightness level must be between 0 and 100")]
    [else
     (set-led-pwm (/ level 100.0))]))
```

### Resource Management

```scheme
;; Monitor memory usage
(define (check-memory-usage)
  (let ((memory (memory-info)))
    (when (< (get memory 'free) 1000)  ; Less than 1KB free
      (log "warn" "Low memory, cleaning up")
      (cleanup-old-data)
      (force-garbage-collection))))

;; Clean up resources
(define (cleanup-old-data)
  (delete "old-sensor-data")
  (delete "temporary-calculations")
  (log "info" "Cleanup completed"))

;; Limit program execution time
(define (with-timeout timeout-secs thunk)
  (let ((start-time (current-time)))
    (define (check-timeout)
      (when (> (- (current-time) start-time) timeout-secs)
        (error "Operation timed out")))
    
    (set-timer "timeout-check" 1 check-timeout)
    (let ((result (thunk)))
      (cancel-timer "timeout-check")
      result)))
```

### Security Considerations

```scheme
;; Validate command sources
(define authorized-topics '("commands/admin" "commands/operator"))

(register-event-handler "mqtt-message"
  (lambda (topic payload)
    (if (member topic authorized-topics)
        (handle-authorized-command topic payload)
        (log "warn" (format "Unauthorized command from topic: ~a" topic)))))

;; Sanitize inputs
(define (sanitize-string input)
  (string-filter (lambda (c)
                   (or (char-alphabetic? c)
                       (char-numeric? c)
                       (char=? c #\-)))
                 input))

;; Rate limiting
(define command-timestamps '())

(define (rate-limit-check)
  (let ((now (current-time)))
    (set! command-timestamps 
          (filter (lambda (ts) (< (- now ts) 60))  ; Keep last minute
                  command-timestamps))
    (< (length command-timestamps) 10)))  ; Max 10 commands per minute
```

---

## Debugging and Testing

### Debug Output

```scheme
;; Debug logging
(define debug-enabled #t)

(define (debug-log message)
  (when debug-enabled
    (log "debug" (format "[DEBUG] ~a" message))))

;; Trace function calls
(define (trace-call func-name args result)
  (debug-log (format "~a(~a) -> ~a" func-name args result)))

;; Example usage
(define (traced-sensor-read)
  (let ((result (read-sensors)))
    (trace-call "read-sensors" '() result)
    result))
```

### Testing Utilities

```scheme
;; Simple assertion framework
(define (assert condition message)
  (unless condition
    (log "error" (format "Assertion failed: ~a" message))
    (error message)))

;; Test runner
(define (run-test test-name test-func)
  (log "info" (format "Running test: ~a" test-name))
  (guard (ex
          [else
           (log "error" (format "Test ~a failed: ~a" test-name ex))
           #f])
    (test-func)
    (log "info" (format "Test ~a passed" test-name))
    #t))

;; Example tests
(define (test-led-control)
  (led-off)
  (assert (not (led-state)) "LED should be off")
  (led-on)
  (assert (led-state) "LED should be on")
  (led-off))

(define (test-sensor-reading)
  (let ((sensors (read-sensors)))
    (assert (list? sensors) "Sensors should return a list")
    (assert (assoc 'temperature sensors) "Should have temperature reading")))

;; Run all tests
(define (run-all-tests)
  (let ((tests '(("LED Control" test-led-control)
                ("Sensor Reading" test-sensor-reading))))
    (let ((results (map (lambda (test)
                         (run-test (car test) (cadr test)))
                       tests)))
      (let ((passed (length (filter identity results)))
            (total (length results)))
        (log "info" (format "Tests: ~a/~a passed" passed total))))))
```

---

## Deployment and Updates

### Program Packaging

Steel programs are deployed as `.scm` files through AWS IoT Core. Programs can be:

1. **Uploaded to S3**: For large programs or batch deployments
2. **Sent via MQTT**: For small programs or immediate updates
3. **Updated via Device Shadow**: For configuration-driven deployments

### Version Management

```scheme
;; Include version information in programs
(define program-version "1.2.0")
(define program-name "sensor-monitor")

(log "info" (format "Starting ~a v~a" program-name program-version))

;; Report version to cloud
(shadow-update "program-info" 
               (json-encode '((name . program-name)
                            (version . program-version)
                            (started . (current-time)))))
```

### Hot Updates

Programs can be updated without device restart:

```scheme
;; Check for program updates
(register-event-handler "program-update"
  (lambda (new-program)
    (log "info" "Received program update")
    (stop-current-program)
    (load-and-execute new-program)))

;; Graceful shutdown
(define (stop-current-program)
  (set! monitoring-active #f)
  (cancel-all-timers)
  (log "info" "Program stopped gracefully"))
```

---

## Troubleshooting

### Common Issues

#### Program Won't Load
- Check syntax with Steel validator
- Verify program size limits
- Check device memory availability

#### Runtime Errors
- Review error logs in CloudWatch
- Check for infinite loops or recursion
- Verify API usage and parameters

#### Performance Issues
- Monitor memory usage
- Check for blocking operations
- Optimize frequent operations

### Diagnostic Tools

```scheme
;; System diagnostics
(define (system-diagnostics)
  (let ((device (device-info))
        (memory (memory-info))
        (uptime (uptime)))
    (log "info" (format "Device: ~a" (json-encode device)))
    (log "info" (format "Memory: ~a" (json-encode memory)))
    (log "info" (format "Uptime: ~a seconds" uptime))))

;; Performance profiling
(define (profile-operation name operation)
  (let ((start-time (current-time)))
    (let ((result (operation)))
      (let ((elapsed (- (current-time) start-time)))
        (log "info" (format "~a took ~a seconds" name elapsed))
        result))))
```

---

## API Reference Summary

### Hardware Functions
- `(led-on)` - Turn LED on
- `(led-off)` - Turn LED off  
- `(led-state)` - Get LED state
- `(sleep seconds)` - Sleep for specified time
- `(read-sensors)` - Read all sensor data
- `(device-info)` - Get device information
- `(uptime)` - Get system uptime
- `(memory-info)` - Get memory information

### Cloud Functions
- `(mqtt-publish topic payload)` - Publish MQTT message
- `(shadow-update key value)` - Update device shadow
- `(log level message)` - Write log message

### Storage Functions
- `(store key value)` - Store data persistently
- `(load key)` - Load stored data
- `(delete key)` - Delete stored data

### Timer Functions
- `(set-timer name seconds callback)` - Set timer
- `(cancel-timer name)` - Cancel timer
- `(schedule cron-expr callback)` - Schedule recurring task

### Event Functions
- `(register-event-handler event callback)` - Register event handler
- `(emit-event event data)` - Emit custom event

### Utility Functions
- `(current-time)` - Get current timestamp
- `(json-encode data)` - Encode data as JSON
- `(json-decode string)` - Decode JSON string
- `(format template args...)` - Format string

---

*This guide covers the essential aspects of Steel programming for the AWS IoT Steel System. For additional examples and advanced topics, see the examples directory and API documentation.*