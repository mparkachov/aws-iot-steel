use crate::{SystemError, SystemResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Operational runbooks and troubleshooting guide system
pub struct OperationalRunbooks {
    runbooks: HashMap<String, Runbook>,
    troubleshooting_guides: HashMap<String, TroubleshootingGuide>,
    incident_procedures: HashMap<String, IncidentProcedure>,
}

/// Runbook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: RunbookCategory,
    pub severity: Severity,
    pub steps: Vec<RunbookStep>,
    pub prerequisites: Vec<String>,
    pub estimated_time: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
    pub tags: Vec<String>,
}

/// Runbook step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStep {
    pub step_number: usize,
    pub title: String,
    pub description: String,
    pub action_type: ActionType,
    pub commands: Vec<String>,
    pub expected_output: Option<String>,
    pub validation: Option<String>,
    pub rollback_commands: Vec<String>,
    pub notes: Vec<String>,
}

/// Runbook categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunbookCategory {
    DeviceProvisioning,
    CertificateManagement,
    NetworkTroubleshooting,
    PerformanceOptimization,
    SecurityIncident,
    DataRecovery,
    SystemMaintenance,
    Monitoring,
}

/// Action types for runbook steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    Command,
    Verification,
    Configuration,
    Monitoring,
    Manual,
    Automated,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Troubleshooting guide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleshootingGuide {
    pub id: String,
    pub title: String,
    pub description: String,
    pub symptoms: Vec<String>,
    pub common_causes: Vec<String>,
    pub diagnostic_steps: Vec<DiagnosticStep>,
    pub solutions: Vec<Solution>,
    pub related_runbooks: Vec<String>,
    pub escalation_path: Vec<String>,
}

/// Diagnostic step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticStep {
    pub step_number: usize,
    pub description: String,
    pub command: Option<String>,
    pub expected_result: String,
    pub interpretation: String,
}

/// Solution definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub id: String,
    pub title: String,
    pub description: String,
    pub steps: Vec<String>,
    pub success_criteria: Vec<String>,
    pub risk_level: RiskLevel,
}

/// Risk levels for solutions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Incident procedure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentProcedure {
    pub id: String,
    pub title: String,
    pub incident_type: IncidentType,
    pub severity_matrix: Vec<SeverityMapping>,
    pub response_steps: Vec<ResponseStep>,
    pub escalation_matrix: Vec<EscalationLevel>,
    pub communication_plan: CommunicationPlan,
}

/// Incident types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IncidentType {
    SecurityBreach,
    ServiceOutage,
    DataLoss,
    PerformanceDegradation,
    CertificateExpiry,
    NetworkFailure,
}

/// Severity mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityMapping {
    pub criteria: String,
    pub severity: Severity,
    pub response_time: String,
}

/// Response step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStep {
    pub step_number: usize,
    pub title: String,
    pub description: String,
    pub responsible_role: String,
    pub time_limit: Option<String>,
    pub actions: Vec<String>,
}

/// Escalation level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: usize,
    pub role: String,
    pub contact_method: String,
    pub escalation_criteria: String,
    pub time_threshold: String,
}

/// Communication plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPlan {
    pub internal_channels: Vec<String>,
    pub external_channels: Vec<String>,
    pub update_frequency: String,
    pub stakeholders: Vec<String>,
}

impl Default for OperationalRunbooks {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationalRunbooks {
    /// Create new operational runbooks system
    pub fn new() -> Self {
        let mut system = Self {
            runbooks: HashMap::new(),
            troubleshooting_guides: HashMap::new(),
            incident_procedures: HashMap::new(),
        };

        system.initialize_default_runbooks();
        system.initialize_troubleshooting_guides();
        system.initialize_incident_procedures();
        system
    }

    /// Get runbook by ID
    pub fn get_runbook(&self, id: &str) -> Option<&Runbook> {
        self.runbooks.get(id)
    }

    /// Get troubleshooting guide by ID
    pub fn get_troubleshooting_guide(&self, id: &str) -> Option<&TroubleshootingGuide> {
        self.troubleshooting_guides.get(id)
    }

    /// Get incident procedure by ID
    pub fn get_incident_procedure(&self, id: &str) -> Option<&IncidentProcedure> {
        self.incident_procedures.get(id)
    }

    /// List runbooks by category
    pub fn list_runbooks_by_category(&self, category: &RunbookCategory) -> Vec<&Runbook> {
        self.runbooks
            .values()
            .filter(|runbook| runbook.category == *category)
            .collect()
    }

    /// Search runbooks by keywords
    pub fn search_runbooks(&self, keywords: &[String]) -> Vec<&Runbook> {
        self.runbooks
            .values()
            .filter(|runbook| {
                keywords.iter().any(|keyword| {
                    runbook
                        .title
                        .to_lowercase()
                        .contains(&keyword.to_lowercase())
                        || runbook
                            .description
                            .to_lowercase()
                            .contains(&keyword.to_lowercase())
                        || runbook
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&keyword.to_lowercase()))
                })
            })
            .collect()
    }

    /// Initialize default runbooks
    fn initialize_default_runbooks(&mut self) {
        // Device provisioning runbook
        let device_provisioning = Runbook {
            id: "device-provisioning".to_string(),
            title: "Device Provisioning Procedure".to_string(),
            description: "Complete procedure for provisioning new IoT devices".to_string(),
            category: RunbookCategory::DeviceProvisioning,
            severity: Severity::Medium,
            steps: vec![
                RunbookStep {
                    step_number: 1,
                    title: "Verify device requirements".to_string(),
                    description: "Check device specifications and compatibility".to_string(),
                    action_type: ActionType::Verification,
                    commands: vec!["steel-dev-tools validate --file device-config.json".to_string()],
                    expected_output: Some("Validation: PASSED".to_string()),
                    validation: Some("Ensure all required fields are present".to_string()),
                    rollback_commands: vec![],
                    notes: vec!["Document device serial number".to_string()],
                },
                RunbookStep {
                    step_number: 2,
                    title: "Generate device certificate".to_string(),
                    description: "Create and configure device certificates".to_string(),
                    action_type: ActionType::Command,
                    commands: vec![
                        "aws iot create-keys-and-certificate --set-as-active".to_string()
                    ],
                    expected_output: Some("Certificate created successfully".to_string()),
                    validation: Some("Verify certificate is active".to_string()),
                    rollback_commands: vec![
                        "aws iot delete-certificate --certificate-id <cert-id>".to_string(),
                    ],
                    notes: vec!["Store private key securely".to_string()],
                },
            ],
            prerequisites: vec![
                "AWS CLI configured".to_string(),
                "Device information available".to_string(),
            ],
            estimated_time: "15-30 minutes".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: "1.0.0".to_string(),
            tags: vec!["provisioning".to_string(), "certificates".to_string()],
        };

        self.runbooks
            .insert(device_provisioning.id.clone(), device_provisioning);
    }

    /// Initialize troubleshooting guides
    fn initialize_troubleshooting_guides(&mut self) {
        let connection_issues = TroubleshootingGuide {
            id: "connection-issues".to_string(),
            title: "Device Connection Issues".to_string(),
            description: "Troubleshoot devices that cannot connect to AWS IoT".to_string(),
            symptoms: vec![
                "Device shows as offline in console".to_string(),
                "Connection timeout errors".to_string(),
                "Certificate authentication failures".to_string(),
            ],
            common_causes: vec![
                "Expired or invalid certificates".to_string(),
                "Network connectivity issues".to_string(),
                "Incorrect endpoint configuration".to_string(),
                "Policy restrictions".to_string(),
            ],
            diagnostic_steps: vec![
                DiagnosticStep {
                    step_number: 1,
                    description: "Check certificate status".to_string(),
                    command: Some(
                        "aws iot describe-certificate --certificate-id <cert-id>".to_string(),
                    ),
                    expected_result: "Certificate status: ACTIVE".to_string(),
                    interpretation: "If inactive, certificate needs to be activated or replaced"
                        .to_string(),
                },
                DiagnosticStep {
                    step_number: 2,
                    description: "Test network connectivity".to_string(),
                    command: Some("ping <iot-endpoint>".to_string()),
                    expected_result: "Successful ping responses".to_string(),
                    interpretation: "If ping fails, check network configuration and firewall rules"
                        .to_string(),
                },
            ],
            solutions: vec![Solution {
                id: "cert-renewal".to_string(),
                title: "Renew expired certificate".to_string(),
                description: "Generate new certificate and update device".to_string(),
                steps: vec![
                    "Generate new certificate".to_string(),
                    "Update device configuration".to_string(),
                    "Test connection".to_string(),
                ],
                success_criteria: vec!["Device shows as online".to_string()],
                risk_level: RiskLevel::Low,
            }],
            related_runbooks: vec!["device-provisioning".to_string()],
            escalation_path: vec!["L2 Support".to_string(), "Engineering Team".to_string()],
        };

        self.troubleshooting_guides
            .insert(connection_issues.id.clone(), connection_issues);
    }

    /// Initialize incident procedures
    fn initialize_incident_procedures(&mut self) {
        let security_incident = IncidentProcedure {
            id: "security-incident".to_string(),
            title: "Security Incident Response".to_string(),
            incident_type: IncidentType::SecurityBreach,
            severity_matrix: vec![
                SeverityMapping {
                    criteria: "Unauthorized access to single device".to_string(),
                    severity: Severity::Medium,
                    response_time: "4 hours".to_string(),
                },
                SeverityMapping {
                    criteria: "Unauthorized access to multiple devices or sensitive data"
                        .to_string(),
                    severity: Severity::High,
                    response_time: "1 hour".to_string(),
                },
            ],
            response_steps: vec![ResponseStep {
                step_number: 1,
                title: "Immediate containment".to_string(),
                description: "Isolate affected devices and systems".to_string(),
                responsible_role: "Security Team".to_string(),
                time_limit: Some("15 minutes".to_string()),
                actions: vec![
                    "Disable affected device certificates".to_string(),
                    "Block suspicious IP addresses".to_string(),
                ],
            }],
            escalation_matrix: vec![EscalationLevel {
                level: 1,
                role: "Security Team Lead".to_string(),
                contact_method: "Phone + Slack".to_string(),
                escalation_criteria: "High severity incident".to_string(),
                time_threshold: "30 minutes".to_string(),
            }],
            communication_plan: CommunicationPlan {
                internal_channels: vec!["#security-incidents".to_string()],
                external_channels: vec!["Customer notifications".to_string()],
                update_frequency: "Every 30 minutes".to_string(),
                stakeholders: vec!["Security Team".to_string(), "Management".to_string()],
            },
        };

        self.incident_procedures
            .insert(security_incident.id.clone(), security_incident);
    }

    /// Execute runbook step
    pub fn execute_runbook_step(
        &self,
        runbook_id: &str,
        step_number: usize,
    ) -> SystemResult<StepExecutionResult> {
        let runbook = self.get_runbook(runbook_id).ok_or_else(|| {
            SystemError::Configuration(format!("Runbook not found: {}", runbook_id))
        })?;

        let step = runbook
            .steps
            .iter()
            .find(|s| s.step_number == step_number)
            .ok_or_else(|| {
                SystemError::Configuration(format!(
                    "Step {} not found in runbook {}",
                    step_number, runbook_id
                ))
            })?;

        info!("Executing runbook step: {} - {}", runbook.title, step.title);

        // In a real implementation, this would execute the actual commands
        // For now, we'll simulate the execution
        let execution_result = StepExecutionResult {
            step_number,
            success: true,
            output: step.expected_output.clone().unwrap_or_default(),
            execution_time: std::time::Duration::from_secs(1),
            notes: vec!["Simulated execution".to_string()],
        };

        Ok(execution_result)
    }

    /// Generate runbook execution report
    pub fn generate_execution_report(
        &self,
        runbook_id: &str,
        results: &[StepExecutionResult],
    ) -> SystemResult<ExecutionReport> {
        let runbook = self.get_runbook(runbook_id).ok_or_else(|| {
            SystemError::Configuration(format!("Runbook not found: {}", runbook_id))
        })?;

        let total_steps = runbook.steps.len();
        let completed_steps = results.len();
        let successful_steps = results.iter().filter(|r| r.success).count();
        let failed_steps = results.iter().filter(|r| !r.success).count();

        let total_execution_time: std::time::Duration =
            results.iter().map(|r| r.execution_time).sum();

        let report = ExecutionReport {
            runbook_id: runbook_id.to_string(),
            runbook_title: runbook.title.clone(),
            execution_date: Utc::now(),
            total_steps,
            completed_steps,
            successful_steps,
            failed_steps,
            total_execution_time,
            step_results: results.to_vec(),
            overall_success: failed_steps == 0 && completed_steps == total_steps,
            recommendations: self.generate_recommendations(results),
        };

        Ok(report)
    }

    /// Generate recommendations based on execution results
    fn generate_recommendations(&self, results: &[StepExecutionResult]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let failed_steps: Vec<_> = results.iter().filter(|r| !r.success).collect();
        if !failed_steps.is_empty() {
            recommendations.push(format!(
                "Review and retry {} failed steps",
                failed_steps.len()
            ));
        }

        let slow_steps: Vec<_> = results
            .iter()
            .filter(|r| r.execution_time > std::time::Duration::from_secs(60))
            .collect();
        if !slow_steps.is_empty() {
            recommendations.push("Consider optimizing slow-running steps".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("All steps completed successfully".to_string());
        }

        recommendations
    }
}

/// Step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    pub step_number: usize,
    pub success: bool,
    pub output: String,
    pub execution_time: std::time::Duration,
    pub notes: Vec<String>,
}

/// Execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub runbook_id: String,
    pub runbook_title: String,
    pub execution_date: DateTime<Utc>,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub successful_steps: usize,
    pub failed_steps: usize,
    pub total_execution_time: std::time::Duration,
    pub step_results: Vec<StepExecutionResult>,
    pub overall_success: bool,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runbook_creation() {
        let runbooks = OperationalRunbooks::new();
        assert!(!runbooks.runbooks.is_empty());
        assert!(!runbooks.troubleshooting_guides.is_empty());
        assert!(!runbooks.incident_procedures.is_empty());
    }

    #[test]
    fn test_runbook_search() {
        let runbooks = OperationalRunbooks::new();
        let results = runbooks.search_runbooks(&["provisioning".to_string()]);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_runbook_execution() {
        let runbooks = OperationalRunbooks::new();
        let result = runbooks.execute_runbook_step("device-provisioning", 1);
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}
