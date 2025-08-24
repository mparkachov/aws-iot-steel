use crate::{
    device_provisioning::{DeviceRecord, DeviceStatus, FleetStatistics},
    SystemError, SystemResult,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, info};

/// Monitoring dashboard for device fleet management
pub struct MonitoringDashboard {
    metrics_collector: MetricsCollector,
    alert_manager: AlertManager,
    #[allow(dead_code)]
    dashboard_config: DashboardConfig,
}

/// Metrics collector for gathering device and system metrics
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    device_metrics: HashMap<String, DeviceMetrics>,
    system_metrics: SystemMetrics,
    metric_history: VecDeque<MetricSnapshot>,
    max_history_size: usize,
}

/// Device-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMetrics {
    pub device_id: String,
    pub last_seen: Option<DateTime<Utc>>,
    pub connection_status: ConnectionStatus,
    pub message_count: u64,
    pub error_count: u64,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub battery_level: Option<f64>,
    pub signal_strength: Option<f64>,
    pub firmware_version: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub temperature: Option<f64>,
    pub last_error: Option<String>,
    pub last_error_time: Option<DateTime<Utc>>,
}

/// System-wide metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_devices: usize,
    pub online_devices: usize,
    pub offline_devices: usize,
    pub total_messages: u64,
    pub messages_per_minute: f64,
    pub error_rate: f64,
    pub average_response_time: f64,
    pub system_uptime: Duration,
    pub last_updated: DateTime<Utc>,
}

/// Connection status for devices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConnectionStatus {
    Online,
    Offline,
    Connecting,
    Disconnecting,
    Unknown,
}

/// Metric snapshot for historical tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub timestamp: DateTime<Utc>,
    pub system_metrics: SystemMetrics,
    pub device_count_by_status: HashMap<ConnectionStatus, usize>,
    pub top_errors: Vec<ErrorSummary>,
}

/// Error summary for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub error_type: String,
    pub count: u64,
    pub last_occurrence: DateTime<Utc>,
    pub affected_devices: Vec<String>,
}

/// Alert manager for monitoring alerts
#[derive(Debug, Clone)]
pub struct AlertManager {
    alert_rules: Vec<AlertRule>,
    active_alerts: HashMap<String, Alert>,
    alert_history: VecDeque<Alert>,
    max_history_size: usize,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_minutes: u32,
    pub notification_channels: Vec<String>,
}

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    DeviceOffline {
        device_id: String,
        threshold_minutes: u32,
    },
    HighErrorRate {
        threshold_percentage: f64,
        window_minutes: u32,
    },
    LowBatteryLevel {
        threshold_percentage: f64,
    },
    HighCpuUsage {
        threshold_percentage: f64,
    },
    HighMemoryUsage {
        threshold_percentage: f64,
    },
    HighTemperature {
        threshold_celsius: f64,
    },
    MessageRateAnomaly {
        deviation_percentage: f64,
    },
    CertificateExpiring {
        days_before_expiry: u32,
    },
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Active alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub device_id: Option<String>,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub status: AlertStatus,
    pub metadata: HashMap<String, String>,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
}

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub refresh_interval_seconds: u64,
    pub metric_retention_hours: u32,
    pub alert_retention_days: u32,
    pub enable_real_time_updates: bool,
    pub max_devices_per_page: usize,
}

/// Dashboard view data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardView {
    pub system_overview: SystemOverview,
    pub device_summary: DeviceSummary,
    pub recent_alerts: Vec<Alert>,
    pub performance_metrics: PerformanceMetrics,
    pub fleet_health: FleetHealth,
    pub last_updated: DateTime<Utc>,
}

/// System overview for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    pub total_devices: usize,
    pub online_devices: usize,
    pub offline_devices: usize,
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub message_throughput: f64,
    pub error_rate: f64,
    pub system_health_score: f64, // 0-100
}

/// Device summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSummary {
    pub devices_by_status: HashMap<ConnectionStatus, usize>,
    pub devices_by_type: HashMap<String, usize>,
    pub top_devices_by_messages: Vec<DeviceMessageStats>,
    pub devices_with_errors: Vec<DeviceErrorStats>,
    pub low_battery_devices: Vec<String>,
}

/// Device message statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMessageStats {
    pub device_id: String,
    pub message_count: u64,
    pub messages_per_hour: f64,
}

/// Device error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceErrorStats {
    pub device_id: String,
    pub error_count: u64,
    pub error_rate: f64,
    pub last_error: Option<String>,
}

/// Performance metrics for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time: f64,
    pub message_processing_rate: f64,
    pub peak_concurrent_connections: usize,
    pub bandwidth_usage: f64, // MB/hour
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

/// Fleet health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetHealth {
    pub overall_score: f64, // 0-100
    pub connectivity_score: f64,
    pub performance_score: f64,
    pub security_score: f64,
    pub maintenance_score: f64,
    pub recommendations: Vec<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 30,
            metric_retention_hours: 24,
            alert_retention_days: 30,
            enable_real_time_updates: true,
            max_devices_per_page: 50,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            device_metrics: HashMap::new(),
            system_metrics: SystemMetrics {
                total_devices: 0,
                online_devices: 0,
                offline_devices: 0,
                total_messages: 0,
                messages_per_minute: 0.0,
                error_rate: 0.0,
                average_response_time: 0.0,
                system_uptime: Duration::zero(),
                last_updated: Utc::now(),
            },
            metric_history: VecDeque::new(),
            max_history_size: 1000,
        }
    }

    /// Update device metrics
    pub fn update_device_metrics(&mut self, device_id: &str, metrics: DeviceMetrics) {
        self.device_metrics.insert(device_id.to_string(), metrics);
        self.update_system_metrics();
    }

    /// Record device message
    pub fn record_device_message(&mut self, device_id: &str) {
        if let Some(metrics) = self.device_metrics.get_mut(device_id) {
            metrics.message_count += 1;
            metrics.last_seen = Some(Utc::now());
            metrics.connection_status = ConnectionStatus::Online;
        }
        self.system_metrics.total_messages += 1;
    }

    /// Record device error
    pub fn record_device_error(&mut self, device_id: &str, error_message: &str) {
        if let Some(metrics) = self.device_metrics.get_mut(device_id) {
            metrics.error_count += 1;
            metrics.last_error = Some(error_message.to_string());
            metrics.last_error_time = Some(Utc::now());
        }
    }

    /// Update device connection status
    pub fn update_device_connection(&mut self, device_id: &str, status: ConnectionStatus) {
        if let Some(metrics) = self.device_metrics.get_mut(device_id) {
            metrics.connection_status = status.clone();
            if status == ConnectionStatus::Online {
                metrics.last_seen = Some(Utc::now());
            }
        }
        self.update_system_metrics();
    }

    /// Get device metrics
    pub fn get_device_metrics(&self, device_id: &str) -> Option<&DeviceMetrics> {
        self.device_metrics.get(device_id)
    }

    /// Get system metrics
    pub fn get_system_metrics(&self) -> &SystemMetrics {
        &self.system_metrics
    }

    /// Take metric snapshot
    pub fn take_snapshot(&mut self) {
        let mut device_count_by_status = HashMap::new();
        for metrics in self.device_metrics.values() {
            *device_count_by_status
                .entry(metrics.connection_status.clone())
                .or_insert(0) += 1;
        }

        let snapshot = MetricSnapshot {
            timestamp: Utc::now(),
            system_metrics: self.system_metrics.clone(),
            device_count_by_status,
            top_errors: self.get_top_errors(5),
        };

        self.metric_history.push_back(snapshot);
        if self.metric_history.len() > self.max_history_size {
            self.metric_history.pop_front();
        }
    }

    /// Get metric history
    pub fn get_metric_history(&self, hours: u32) -> Vec<&MetricSnapshot> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours as i64);
        self.metric_history
            .iter()
            .filter(|snapshot| snapshot.timestamp >= cutoff_time)
            .collect()
    }

    /// Update system-wide metrics
    fn update_system_metrics(&mut self) {
        self.system_metrics.total_devices = self.device_metrics.len();
        self.system_metrics.online_devices = self
            .device_metrics
            .values()
            .filter(|m| m.connection_status == ConnectionStatus::Online)
            .count();
        self.system_metrics.offline_devices =
            self.system_metrics.total_devices - self.system_metrics.online_devices;

        // Calculate error rate
        let total_messages: u64 = self.device_metrics.values().map(|m| m.message_count).sum();
        let total_errors: u64 = self.device_metrics.values().map(|m| m.error_count).sum();

        if total_messages > 0 {
            self.system_metrics.error_rate = (total_errors as f64 / total_messages as f64) * 100.0;
        }

        self.system_metrics.last_updated = Utc::now();
    }

    /// Get top errors
    fn get_top_errors(&self, limit: usize) -> Vec<ErrorSummary> {
        let mut error_counts: HashMap<String, (u64, DateTime<Utc>, Vec<String>)> = HashMap::new();

        for (device_id, metrics) in &self.device_metrics {
            if let Some(error) = &metrics.last_error {
                let entry =
                    error_counts
                        .entry(error.clone())
                        .or_insert((0, Utc::now(), Vec::new()));
                entry.0 += metrics.error_count;
                if let Some(error_time) = metrics.last_error_time {
                    if error_time > entry.1 {
                        entry.1 = error_time;
                    }
                }
                entry.2.push(device_id.clone());
            }
        }

        let mut errors: Vec<ErrorSummary> = error_counts
            .into_iter()
            .map(
                |(error_type, (count, last_occurrence, affected_devices))| ErrorSummary {
                    error_type,
                    count,
                    last_occurrence,
                    affected_devices,
                },
            )
            .collect();

        errors.sort_by(|a, b| b.count.cmp(&a.count));
        errors.truncate(limit);
        errors
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alert_rules: Vec::new(),
            active_alerts: HashMap::new(),
            alert_history: VecDeque::new(),
            max_history_size: 1000,
        }
    }

    /// Add alert rule
    pub fn add_alert_rule(&mut self, rule: AlertRule) {
        info!("Adding alert rule: {} ({})", rule.name, rule.id);
        self.alert_rules.push(rule);
    }

    /// Remove alert rule
    pub fn remove_alert_rule(&mut self, rule_id: &str) -> SystemResult<()> {
        let initial_len = self.alert_rules.len();
        self.alert_rules.retain(|rule| rule.id != rule_id);

        if self.alert_rules.len() < initial_len {
            info!("Removed alert rule: {}", rule_id);
            Ok(())
        } else {
            Err(SystemError::Configuration(format!(
                "Alert rule not found: {}",
                rule_id
            )))
        }
    }

    /// Evaluate alert rules against current metrics
    pub fn evaluate_alerts(
        &mut self,
        metrics: &MetricsCollector,
        devices: &[DeviceRecord],
    ) -> Vec<Alert> {
        let mut new_alerts = Vec::new();

        for rule in &self.alert_rules {
            if !rule.enabled {
                continue;
            }

            // Check if rule is in cooldown
            if let Some(existing_alert) = self.active_alerts.get(&rule.id) {
                let cooldown_duration = chrono::Duration::minutes(rule.cooldown_minutes as i64);
                if Utc::now() - existing_alert.triggered_at < cooldown_duration {
                    continue;
                }
            }

            if let Some(alert) = self.evaluate_rule(rule, metrics, devices) {
                new_alerts.push(alert);
            }
        }

        // Add new alerts to active alerts
        for alert in &new_alerts {
            self.active_alerts
                .insert(alert.rule_id.clone(), alert.clone());
            self.alert_history.push_back(alert.clone());
        }

        // Cleanup old history
        if self.alert_history.len() > self.max_history_size {
            self.alert_history.pop_front();
        }

        new_alerts
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> SystemResult<()> {
        if let Some(alert) = self.active_alerts.values_mut().find(|a| a.id == alert_id) {
            alert.acknowledged_at = Some(Utc::now());
            alert.status = AlertStatus::Acknowledged;
            info!("Alert acknowledged: {}", alert_id);
            Ok(())
        } else {
            Err(SystemError::Configuration(format!(
                "Alert not found: {}",
                alert_id
            )))
        }
    }

    /// Resolve an alert
    pub fn resolve_alert(&mut self, alert_id: &str) -> SystemResult<()> {
        if let Some(alert) = self.active_alerts.values_mut().find(|a| a.id == alert_id) {
            alert.resolved_at = Some(Utc::now());
            alert.status = AlertStatus::Resolved;
            info!("Alert resolved: {}", alert_id);
            Ok(())
        } else {
            Err(SystemError::Configuration(format!(
                "Alert not found: {}",
                alert_id
            )))
        }
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        self.active_alerts
            .values()
            .filter(|alert| alert.status == AlertStatus::Active)
            .collect()
    }

    /// Get alert history
    pub fn get_alert_history(&self, hours: u32) -> Vec<&Alert> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours as i64);
        self.alert_history
            .iter()
            .filter(|alert| alert.triggered_at >= cutoff_time)
            .collect()
    }

    /// Evaluate a single alert rule
    fn evaluate_rule(
        &self,
        rule: &AlertRule,
        metrics: &MetricsCollector,
        devices: &[DeviceRecord],
    ) -> Option<Alert> {
        match &rule.condition {
            AlertCondition::DeviceOffline {
                device_id,
                threshold_minutes,
            } => {
                if let Some(device_metrics) = metrics.get_device_metrics(device_id) {
                    if device_metrics.connection_status == ConnectionStatus::Offline {
                        if let Some(last_seen) = device_metrics.last_seen {
                            let offline_duration = Utc::now() - last_seen;
                            if offline_duration
                                > chrono::Duration::minutes(*threshold_minutes as i64)
                            {
                                return Some(Alert {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    rule_id: rule.id.clone(),
                                    device_id: Some(device_id.clone()),
                                    severity: rule.severity.clone(),
                                    title: format!("Device Offline: {}", device_id),
                                    description: format!(
                                        "Device {} has been offline for {} minutes",
                                        device_id,
                                        offline_duration.num_minutes()
                                    ),
                                    triggered_at: Utc::now(),
                                    acknowledged_at: None,
                                    resolved_at: None,
                                    status: AlertStatus::Active,
                                    metadata: HashMap::new(),
                                });
                            }
                        }
                    }
                }
            }

            AlertCondition::HighErrorRate {
                threshold_percentage,
                window_minutes: _,
            } => {
                let system_metrics = metrics.get_system_metrics();
                if system_metrics.error_rate > *threshold_percentage {
                    return Some(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        rule_id: rule.id.clone(),
                        device_id: None,
                        severity: rule.severity.clone(),
                        title: "High Error Rate".to_string(),
                        description: format!(
                            "System error rate is {:.2}%, exceeding threshold of {:.2}%",
                            system_metrics.error_rate, threshold_percentage
                        ),
                        triggered_at: Utc::now(),
                        acknowledged_at: None,
                        resolved_at: None,
                        status: AlertStatus::Active,
                        metadata: HashMap::new(),
                    });
                }
            }

            AlertCondition::LowBatteryLevel {
                threshold_percentage,
            } => {
                for device_metrics in metrics.device_metrics.values() {
                    if let Some(battery_level) = device_metrics.battery_level {
                        if battery_level < *threshold_percentage {
                            return Some(Alert {
                                id: uuid::Uuid::new_v4().to_string(),
                                rule_id: rule.id.clone(),
                                device_id: Some(device_metrics.device_id.clone()),
                                severity: rule.severity.clone(),
                                title: format!("Low Battery: {}", device_metrics.device_id),
                                description: format!(
                                    "Device {} battery level is {:.1}%, below threshold of {:.1}%",
                                    device_metrics.device_id, battery_level, threshold_percentage
                                ),
                                triggered_at: Utc::now(),
                                acknowledged_at: None,
                                resolved_at: None,
                                status: AlertStatus::Active,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }

            AlertCondition::CertificateExpiring { days_before_expiry } => {
                let _expiry_threshold =
                    Utc::now() + chrono::Duration::days(*days_before_expiry as i64);
                for device in devices {
                    // In a real implementation, we would check certificate expiry dates
                    // For now, we'll create a placeholder alert
                    if device.status == DeviceStatus::Active {
                        // Simulate certificate expiry check
                        let device_created_days_ago = (Utc::now() - device.created_at).num_days();
                        if device_created_days_ago > 330 {
                            // Simulate certificates expiring after ~1 year
                            return Some(Alert {
                                id: uuid::Uuid::new_v4().to_string(),
                                rule_id: rule.id.clone(),
                                device_id: Some(device.device_id.clone()),
                                severity: rule.severity.clone(),
                                title: format!("Certificate Expiring: {}", device.device_id),
                                description: format!(
                                    "Certificate for device {} expires within {} days",
                                    device.device_id, days_before_expiry
                                ),
                                triggered_at: Utc::now(),
                                acknowledged_at: None,
                                resolved_at: None,
                                status: AlertStatus::Active,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }

            _ => {
                // Other alert conditions would be implemented here
                debug!("Alert condition not yet implemented: {:?}", rule.condition);
            }
        }

        None
    }
}

impl MonitoringDashboard {
    /// Create a new monitoring dashboard
    pub fn new(config: DashboardConfig) -> Self {
        let mut alert_manager = AlertManager::new();

        // Add default alert rules
        alert_manager.add_alert_rule(AlertRule {
            id: "device-offline".to_string(),
            name: "Device Offline".to_string(),
            description: "Alert when a device goes offline for more than 5 minutes".to_string(),
            condition: AlertCondition::DeviceOffline {
                device_id: "*".to_string(), // Wildcard for all devices
                threshold_minutes: 5,
            },
            severity: AlertSeverity::Warning,
            enabled: true,
            cooldown_minutes: 15,
            notification_channels: vec!["email".to_string()],
        });

        alert_manager.add_alert_rule(AlertRule {
            id: "high-error-rate".to_string(),
            name: "High Error Rate".to_string(),
            description: "Alert when system error rate exceeds 5%".to_string(),
            condition: AlertCondition::HighErrorRate {
                threshold_percentage: 5.0,
                window_minutes: 10,
            },
            severity: AlertSeverity::Critical,
            enabled: true,
            cooldown_minutes: 30,
            notification_channels: vec!["email".to_string(), "slack".to_string()],
        });

        Self {
            metrics_collector: MetricsCollector::new(),
            alert_manager,
            dashboard_config: config,
        }
    }

    /// Update device metrics
    pub fn update_device_metrics(&mut self, device_id: &str, metrics: DeviceMetrics) {
        self.metrics_collector
            .update_device_metrics(device_id, metrics);
    }

    /// Record device activity
    pub fn record_device_message(&mut self, device_id: &str) {
        self.metrics_collector.record_device_message(device_id);
    }

    /// Record device error
    pub fn record_device_error(&mut self, device_id: &str, error_message: &str) {
        self.metrics_collector
            .record_device_error(device_id, error_message);
    }

    /// Update device connection status
    pub fn update_device_connection(&mut self, device_id: &str, status: ConnectionStatus) {
        self.metrics_collector
            .update_device_connection(device_id, status);
    }

    /// Generate dashboard view
    pub fn generate_dashboard_view(
        &mut self,
        devices: &[DeviceRecord],
        fleet_stats: &FleetStatistics,
    ) -> DashboardView {
        // Take metric snapshot
        self.metrics_collector.take_snapshot();

        // Evaluate alerts
        let _new_alerts = self
            .alert_manager
            .evaluate_alerts(&self.metrics_collector, devices);

        let system_metrics = self.metrics_collector.get_system_metrics();
        let active_alerts = self.alert_manager.get_active_alerts();

        // Generate system overview
        let system_overview = SystemOverview {
            total_devices: fleet_stats.total_devices,
            online_devices: system_metrics.online_devices,
            offline_devices: system_metrics.offline_devices,
            active_alerts: active_alerts.len(),
            critical_alerts: active_alerts
                .iter()
                .filter(|alert| alert.severity >= AlertSeverity::Critical)
                .count(),
            message_throughput: system_metrics.messages_per_minute,
            error_rate: system_metrics.error_rate,
            system_health_score: self.calculate_system_health_score(system_metrics, &active_alerts),
        };

        // Generate device summary
        let device_summary = self.generate_device_summary();

        // Generate performance metrics
        let performance_metrics = PerformanceMetrics {
            average_response_time: system_metrics.average_response_time,
            message_processing_rate: system_metrics.messages_per_minute,
            peak_concurrent_connections: system_metrics.online_devices,
            bandwidth_usage: system_metrics.messages_per_minute * 0.5, // Estimate
            cpu_usage: 45.0,                                           // Mock data
            memory_usage: 62.0,                                        // Mock data
        };

        // Generate fleet health
        let fleet_health = self.calculate_fleet_health(fleet_stats, system_metrics, &active_alerts);

        DashboardView {
            system_overview,
            device_summary,
            recent_alerts: active_alerts.into_iter().cloned().collect(),
            performance_metrics,
            fleet_health,
            last_updated: Utc::now(),
        }
    }

    /// Get alert manager
    pub fn get_alert_manager(&mut self) -> &mut AlertManager {
        &mut self.alert_manager
    }

    /// Get metrics collector
    pub fn get_metrics_collector(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    /// Calculate system health score (0-100)
    fn calculate_system_health_score(
        &self,
        system_metrics: &SystemMetrics,
        active_alerts: &[&Alert],
    ) -> f64 {
        let mut score = 100.0;

        // Deduct points for offline devices
        if system_metrics.total_devices > 0 {
            let offline_percentage = (system_metrics.offline_devices as f64
                / system_metrics.total_devices as f64)
                * 100.0;
            score -= offline_percentage * 0.5;
        }

        // Deduct points for error rate
        score -= system_metrics.error_rate * 2.0;

        // Deduct points for active alerts
        for alert in active_alerts {
            match alert.severity {
                AlertSeverity::Info => score -= 1.0,
                AlertSeverity::Warning => score -= 5.0,
                AlertSeverity::Critical => score -= 15.0,
                AlertSeverity::Emergency => score -= 25.0,
            }
        }

        score.clamp(0.0, 100.0)
    }

    /// Generate device summary
    fn generate_device_summary(&self) -> DeviceSummary {
        let mut devices_by_status = HashMap::new();
        let devices_by_type = HashMap::new();
        let mut device_message_stats = Vec::new();
        let mut device_error_stats = Vec::new();
        let mut low_battery_devices = Vec::new();

        for (device_id, metrics) in &self.metrics_collector.device_metrics {
            // Count by status
            *devices_by_status
                .entry(metrics.connection_status.clone())
                .or_insert(0) += 1;

            // Message stats
            device_message_stats.push(DeviceMessageStats {
                device_id: device_id.clone(),
                message_count: metrics.message_count,
                messages_per_hour: metrics.message_count as f64, // Simplified calculation
            });

            // Error stats
            if metrics.error_count > 0 {
                device_error_stats.push(DeviceErrorStats {
                    device_id: device_id.clone(),
                    error_count: metrics.error_count,
                    error_rate: if metrics.message_count > 0 {
                        (metrics.error_count as f64 / metrics.message_count as f64) * 100.0
                    } else {
                        0.0
                    },
                    last_error: metrics.last_error.clone(),
                });
            }

            // Low battery devices
            if let Some(battery_level) = metrics.battery_level {
                if battery_level < 20.0 {
                    low_battery_devices.push(device_id.clone());
                }
            }
        }

        // Sort by message count (top devices)
        device_message_stats.sort_by(|a, b| b.message_count.cmp(&a.message_count));
        device_message_stats.truncate(10);

        // Sort by error count
        device_error_stats.sort_by(|a, b| b.error_count.cmp(&a.error_count));
        device_error_stats.truncate(10);

        DeviceSummary {
            devices_by_status,
            devices_by_type,
            top_devices_by_messages: device_message_stats,
            devices_with_errors: device_error_stats,
            low_battery_devices,
        }
    }

    /// Calculate fleet health assessment
    fn calculate_fleet_health(
        &self,
        fleet_stats: &FleetStatistics,
        system_metrics: &SystemMetrics,
        active_alerts: &[&Alert],
    ) -> FleetHealth {
        // Connectivity score
        let connectivity_score = if fleet_stats.total_devices > 0 {
            (fleet_stats.active_devices as f64 / fleet_stats.total_devices as f64) * 100.0
        } else {
            100.0
        };

        // Performance score (based on error rate and response time)
        let performance_score = (100.0 - system_metrics.error_rate).max(0.0);

        // Security score (based on certificate status)
        let security_score = if fleet_stats.certificates_active > 0 {
            let expired_certs = fleet_stats.certificates_expiring_soon as f64;
            let total_certs = fleet_stats.certificates_active as f64;
            ((total_certs - expired_certs) / total_certs * 100.0).max(0.0)
        } else {
            100.0
        };

        // Maintenance score (based on alerts and device status)
        let maintenance_score = 100.0 - (active_alerts.len() as f64 * 5.0).min(100.0);

        // Overall score
        let overall_score =
            (connectivity_score + performance_score + security_score + maintenance_score) / 4.0;

        // Generate recommendations
        let mut recommendations = Vec::new();
        if connectivity_score < 90.0 {
            recommendations.push("Check network connectivity for offline devices".to_string());
        }
        if performance_score < 95.0 {
            recommendations
                .push("Investigate high error rates and optimize performance".to_string());
        }
        if security_score < 90.0 {
            recommendations.push("Renew expiring certificates".to_string());
        }
        if fleet_stats.certificates_expiring_soon > 0 {
            recommendations.push(format!(
                "{} certificates expiring soon",
                fleet_stats.certificates_expiring_soon
            ));
        }

        FleetHealth {
            overall_score,
            connectivity_score,
            performance_score,
            security_score,
            maintenance_score,
            recommendations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        let device_metrics = DeviceMetrics {
            device_id: "test-device".to_string(),
            last_seen: Some(Utc::now()),
            connection_status: ConnectionStatus::Online,
            message_count: 100,
            error_count: 5,
            cpu_usage: Some(45.0),
            memory_usage: Some(60.0),
            battery_level: Some(85.0),
            signal_strength: Some(-65.0),
            firmware_version: Some("1.0.0".to_string()),
            uptime_seconds: Some(3600),
            temperature: Some(25.0),
            last_error: Some("Connection timeout".to_string()),
            last_error_time: Some(Utc::now()),
        };

        collector.update_device_metrics("test-device", device_metrics);

        let retrieved_metrics = collector.get_device_metrics("test-device").unwrap();
        assert_eq!(retrieved_metrics.device_id, "test-device");
        assert_eq!(retrieved_metrics.message_count, 100);
        assert_eq!(retrieved_metrics.error_count, 5);

        let system_metrics = collector.get_system_metrics();
        assert_eq!(system_metrics.total_devices, 1);
        assert_eq!(system_metrics.online_devices, 1);
    }

    #[test]
    fn test_alert_manager() {
        let mut alert_manager = AlertManager::new();

        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test alert rule".to_string(),
            condition: AlertCondition::HighErrorRate {
                threshold_percentage: 10.0,
                window_minutes: 5,
            },
            severity: AlertSeverity::Warning,
            enabled: true,
            cooldown_minutes: 15,
            notification_channels: vec!["email".to_string()],
        };

        alert_manager.add_alert_rule(rule);
        assert_eq!(alert_manager.alert_rules.len(), 1);

        // Test rule removal
        alert_manager.remove_alert_rule("test-rule").unwrap();
        assert_eq!(alert_manager.alert_rules.len(), 0);
    }

    #[test]
    fn test_dashboard_creation() {
        let config = DashboardConfig::default();
        let dashboard = MonitoringDashboard::new(config);

        assert_eq!(dashboard.alert_manager.alert_rules.len(), 2); // Default rules
        assert_eq!(dashboard.metrics_collector.device_metrics.len(), 0);
    }

    #[test]
    fn test_system_health_calculation() {
        let dashboard = MonitoringDashboard::new(DashboardConfig::default());

        let system_metrics = SystemMetrics {
            total_devices: 10,
            online_devices: 9,
            offline_devices: 1,
            total_messages: 1000,
            messages_per_minute: 50.0,
            error_rate: 2.0,
            average_response_time: 100.0,
            system_uptime: Duration::hours(24),
            last_updated: Utc::now(),
        };

        let active_alerts = vec![];
        let health_score = dashboard.calculate_system_health_score(&system_metrics, &active_alerts);

        assert!(health_score > 90.0); // Should be high with low error rate and few offline devices
        assert!(health_score <= 100.0);
    }
}
