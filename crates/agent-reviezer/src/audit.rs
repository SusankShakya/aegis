//! Audit functionality for the Reviezer agent
//!
//! This module provides audit capabilities for system operations,
//! agent behavior, and security compliance.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::LogEntry;

/// Represents the type of audit being performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditType {
    /// Security audit for checking access controls and permissions
    Security,
    /// Performance audit for checking system efficiency
    Performance,
    /// Compliance audit for checking adherence to policies
    Compliance,
    /// Agent behavior audit for monitoring agent operations
    AgentBehavior,
    /// Custom audit type with a specified name
    Custom(String),
}

/// Severity level of an audit finding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingSeverity {
    /// Informational finding, no action needed
    Info,
    /// Low severity finding, may need attention eventually
    Low,
    /// Medium severity finding, should be addressed
    Medium,
    /// High severity finding, needs prompt attention
    High,
    /// Critical severity finding, requires immediate action
    Critical,
}

/// Represents a single finding in an audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    /// Unique identifier for the finding
    pub id: String,
    /// Short description of the finding
    pub title: String,
    /// Detailed description of the finding
    pub description: String,
    /// Severity level of the finding
    pub severity: FindingSeverity,
    /// When the finding was discovered
    pub timestamp: DateTime<Utc>,
    /// Entity related to the finding (agent ID, service name, etc.)
    pub related_entity: String,
    /// Location or context where the finding was discovered
    pub location: String,
    /// Recommended remediation steps
    pub recommendation: Option<String>,
    /// Additional metadata about the finding
    pub metadata: Option<serde_json::Value>,
}

/// A recommendation from the audit process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecommendation {
    /// Unique identifier for the recommendation
    pub id: String,
    /// Short title for the recommendation
    pub title: String,
    /// Detailed description of the recommendation
    pub description: String,
    /// Priority level (1-5, with 1 being highest)
    pub priority: u8,
    /// Estimated effort to implement (1-5, with 1 being lowest)
    pub effort: u8,
    /// Entity the recommendation applies to
    pub target: String,
    /// Expected benefits of implementing the recommendation
    pub benefits: Vec<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Comprehensive audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// Unique report identifier
    pub id: String,
    /// Type of audit performed
    pub audit_type: AuditType,
    /// Title of the report
    pub title: String,
    /// Target of the audit (system, agent, component)
    pub target: String,
    /// When the report was generated
    pub generated_at: DateTime<Utc>,
    /// Start time of the audit period
    pub period_start: Option<DateTime<Utc>>,
    /// End time of the audit period
    pub period_end: Option<DateTime<Utc>>,
    /// Summary of the audit findings
    pub summary: String,
    /// Detailed findings from the audit
    pub findings: Vec<AuditFinding>,
    /// Recommendations based on the audit
    pub recommendations: Vec<AuditRecommendation>,
    /// Statistics and metrics gathered during the audit
    pub metrics: HashMap<String, serde_json::Value>,
    /// Overall risk score (0-100, with 100 being highest risk)
    pub risk_score: Option<u8>,
}

/// Service for generating audit reports
pub struct AuditService {
    /// Analyzers used for different audit types
    analyzers: HashMap<AuditType, Box<dyn LogAnalyzer>>,
}

/// Trait for analyzing logs and generating audit findings
pub trait LogAnalyzer: Send + Sync {
    /// Analyze a collection of logs and generate findings
    fn analyze(&self, logs: &[LogEntry], context: &AuditContext) -> Vec<AuditFinding>;
    
    /// Get the type of audit this analyzer performs
    fn audit_type(&self) -> AuditType;
}

/// Context information for audit analysis
pub struct AuditContext {
    /// Start time for the audit period
    pub start_time: Option<DateTime<Utc>>,
    /// End time for the audit period
    pub end_time: Option<DateTime<Utc>>,
    /// Target of the audit
    pub target: String,
    /// Additional parameters for the audit
    pub parameters: HashMap<String, String>,
}

impl AuditService {
    /// Create a new audit service with default analyzers
    pub fn new() -> Self {
        let mut analyzers = HashMap::new();
        
        // Add default analyzers
        analyzers.insert(
            AuditType::Security,
            Box::new(SecurityLogAnalyzer::new()) as Box<dyn LogAnalyzer>
        );
        
        analyzers.insert(
            AuditType::Performance,
            Box::new(PerformanceLogAnalyzer::new()) as Box<dyn LogAnalyzer>
        );
        
        analyzers.insert(
            AuditType::Compliance,
            Box::new(ComplianceLogAnalyzer::new()) as Box<dyn LogAnalyzer>
        );
        
        analyzers.insert(
            AuditType::AgentBehavior,
            Box::new(AgentBehaviorAnalyzer::new()) as Box<dyn LogAnalyzer>
        );
        
        Self { analyzers }
    }
    
    /// Register a new log analyzer
    pub fn register_analyzer(&mut self, analyzer: Box<dyn LogAnalyzer>) {
        let audit_type = analyzer.audit_type();
        self.analyzers.insert(audit_type, analyzer);
    }
    
    /// Generate an audit report for the specified type and logs
    pub fn generate_report(
        &self,
        audit_type: AuditType,
        target: String,
        logs: &[LogEntry],
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> AuditReport {
        // Create audit context
        let context = AuditContext {
            start_time,
            end_time,
            target: target.clone(),
            parameters: HashMap::new(),
        };
        
        // Generate findings using the appropriate analyzer
        let findings = if let Some(analyzer) = self.analyzers.get(&audit_type) {
            analyzer.analyze(logs, &context)
        } else {
            // If no specific analyzer is found, use a generic approach
            self.generic_analysis(logs, &context)
        };
        
        // Generate recommendations based on findings
        let recommendations = self.generate_recommendations(&findings);
        
        // Calculate risk score
        let risk_score = self.calculate_risk_score(&findings);
        
        // Build report metrics
        let metrics = self.build_metrics(logs, &findings);
        
        // Create the report
        AuditReport {
            id: generate_report_id(),
            audit_type: audit_type.clone(),
            title: format!("{:?} Audit for {}", audit_type, target),
            target,
            generated_at: Utc::now(),
            period_start: start_time,
            period_end: end_time,
            summary: self.generate_summary(&findings, &recommendations),
            findings,
            recommendations,
            metrics,
            risk_score: Some(risk_score),
        }
    }
    
    /// Generate findings using a generic approach when no specific analyzer is available
    fn generic_analysis(&self, logs: &[LogEntry], context: &AuditContext) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        
        // Look for error and warning logs
        for log in logs {
            if log.level.eq_ignore_ascii_case("error") {
                findings.push(AuditFinding {
                    id: generate_finding_id(),
                    title: "Error Log Detected".to_string(),
                    description: format!("Error log detected: {}", log.message),
                    severity: FindingSeverity::Medium,
                    timestamp: log.timestamp,
                    related_entity: log.source.clone(),
                    location: "Log".to_string(),
                    recommendation: Some("Investigate the cause of this error".to_string()),
                    metadata: log.metadata.clone(),
                });
            } else if log.level.eq_ignore_ascii_case("warn") || log.level.eq_ignore_ascii_case("warning") {
                findings.push(AuditFinding {
                    id: generate_finding_id(),
                    title: "Warning Log Detected".to_string(),
                    description: format!("Warning log detected: {}", log.message),
                    severity: FindingSeverity::Low,
                    timestamp: log.timestamp,
                    related_entity: log.source.clone(),
                    location: "Log".to_string(),
                    recommendation: Some("Review this warning to determine if action is needed".to_string()),
                    metadata: log.metadata.clone(),
                });
            }
        }
        
        findings
    }
    
    /// Generate recommendations based on findings
    fn generate_recommendations(&self, findings: &[AuditFinding]) -> Vec<AuditRecommendation> {
        let mut recommendations = Vec::new();
        
        // Group findings by related entity
        let mut findings_by_entity: HashMap<String, Vec<&AuditFinding>> = HashMap::new();
        for finding in findings {
            findings_by_entity
                .entry(finding.related_entity.clone())
                .or_default()
                .push(finding);
        }
        
        // Create recommendations for entities with critical or high severity findings
        for (entity, entity_findings) in findings_by_entity {
            let critical_count = entity_findings
                .iter()
                .filter(|f| f.severity == FindingSeverity::Critical)
                .count();
            
            let high_count = entity_findings
                .iter()
                .filter(|f| f.severity == FindingSeverity::High)
                .count();
            
            if critical_count > 0 {
                recommendations.push(AuditRecommendation {
                    id: generate_recommendation_id(),
                    title: format!("Address Critical Issues in {}", entity),
                    description: format!(
                        "There are {} critical issues that need immediate attention in {}.",
                        critical_count, entity
                    ),
                    priority: 1,
                    effort: 3,
                    target: entity.clone(),
                    benefits: vec![
                        "Reduce security risks".to_string(),
                        "Prevent potential system failures".to_string(),
                        "Maintain system integrity".to_string(),
                    ],
                    metadata: None,
                });
            }
            
            if high_count > 0 {
                recommendations.push(AuditRecommendation {
                    id: generate_recommendation_id(),
                    title: format!("Address High Severity Issues in {}", entity),
                    description: format!(
                        "There are {} high severity issues that need prompt attention in {}.",
                        high_count, entity
                    ),
                    priority: 2,
                    effort: 3,
                    target: entity,
                    benefits: vec![
                        "Improve system reliability".to_string(),
                        "Reduce potential downtime".to_string(),
                        "Enhance operational efficiency".to_string(),
                    ],
                    metadata: None,
                });
            }
        }
        
        recommendations
    }
    
    /// Calculate a risk score based on findings
    fn calculate_risk_score(&self, findings: &[AuditFinding]) -> u8 {
        // Count findings by severity
        let critical_count = findings.iter().filter(|f| f.severity == FindingSeverity::Critical).count();
        let high_count = findings.iter().filter(|f| f.severity == FindingSeverity::High).count();
        let medium_count = findings.iter().filter(|f| f.severity == FindingSeverity::Medium).count();
        let low_count = findings.iter().filter(|f| f.severity == FindingSeverity::Low).count();
        
        // Weight the findings
        let weighted_sum = (critical_count * 40 + high_count * 20 + medium_count * 5 + low_count * 1) as u32;
        
        // Cap the score at 100
        let score = (weighted_sum as f64 / (findings.len() as f64 + 1.0) * 10.0).min(100.0) as u8;
        
        score
    }
    
    /// Build metrics based on logs and findings
    fn build_metrics(&self, logs: &[LogEntry], findings: &[AuditFinding]) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();
        
        // Total log count
        metrics.insert(
            "log_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(logs.len() as u64))
        );
        
        // Logs by level
        let mut log_levels = HashMap::new();
        for log in logs {
            *log_levels.entry(log.level.clone()).or_insert(0) += 1;
        }
        metrics.insert(
            "logs_by_level".to_string(),
            serde_json::to_value(log_levels).unwrap_or_default()
        );
        
        // Finding counts by severity
        let mut finding_severities = HashMap::new();
        for finding in findings {
            let severity = format!("{:?}", finding.severity);
            *finding_severities.entry(severity).or_insert(0) += 1;
        }
        metrics.insert(
            "findings_by_severity".to_string(),
            serde_json::to_value(finding_severities).unwrap_or_default()
        );
        
        metrics
    }
    
    /// Generate a summary of the audit
    fn generate_summary(&self, findings: &[AuditFinding], recommendations: &[AuditRecommendation]) -> String {
        let critical_count = findings.iter().filter(|f| f.severity == FindingSeverity::Critical).count();
        let high_count = findings.iter().filter(|f| f.severity == FindingSeverity::High).count();
        let medium_count = findings.iter().filter(|f| f.severity == FindingSeverity::Medium).count();
        let low_count = findings.iter().filter(|f| f.severity == FindingSeverity::Low).count();
        let info_count = findings.iter().filter(|f| f.severity == FindingSeverity::Info).count();
        
        format!(
            "Audit completed with {} findings ({} critical, {} high, {} medium, {} low, {} info) and {} recommendations.",
            findings.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            info_count,
            recommendations.len()
        )
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new()
    }
}

/// Security log analyzer for identifying security issues
struct SecurityLogAnalyzer {}

impl SecurityLogAnalyzer {
    /// Create a new security log analyzer
    fn new() -> Self {
        Self {}
    }
}

impl LogAnalyzer for SecurityLogAnalyzer {
    fn analyze(&self, logs: &[LogEntry], context: &AuditContext) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        
        // Look for security-related keywords in logs
        let security_keywords = [
            "permission", "access denied", "unauthorized", "forbidden",
            "security", "breach", "attack", "vulnerability", "exploit",
            "malicious", "injection", "overflow", "credential", "password"
        ];
        
        for log in logs {
            for keyword in &security_keywords {
                if log.message.to_lowercase().contains(&keyword.to_lowercase()) {
                    findings.push(AuditFinding {
                        id: generate_finding_id(),
                        title: format!("Security Issue: {}", keyword),
                        description: format!("Security keyword '{}' found in log: {}", keyword, log.message),
                        severity: match *keyword {
                            "breach" | "attack" | "malicious" | "exploit" => FindingSeverity::Critical,
                            "unauthorized" | "forbidden" | "vulnerability" | "credential" | "password" => FindingSeverity::High,
                            "permission" | "access denied" | "injection" | "overflow" => FindingSeverity::Medium,
                            _ => FindingSeverity::Low,
                        },
                        timestamp: log.timestamp,
                        related_entity: log.source.clone(),
                        location: "Log".to_string(),
                        recommendation: Some(format!("Investigate the {} issue in {}", keyword, log.source)),
                        metadata: log.metadata.clone(),
                    });
                    break; // Only create one finding per log entry
                }
            }
        }
        
        findings
    }
    
    fn audit_type(&self) -> AuditType {
        AuditType::Security
    }
}

/// Performance log analyzer for identifying performance issues
struct PerformanceLogAnalyzer {}

impl PerformanceLogAnalyzer {
    /// Create a new performance log analyzer
    fn new() -> Self {
        Self {}
    }
}

impl LogAnalyzer for PerformanceLogAnalyzer {
    fn analyze(&self, logs: &[LogEntry], context: &AuditContext) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        
        // Look for performance-related keywords in logs
        let performance_keywords = [
            "slow", "timeout", "latency", "delay", "performance",
            "overload", "bottleneck", "cpu", "memory", "leak",
            "resource", "throughput", "response time", "queue"
        ];
        
        for log in logs {
            for keyword in &performance_keywords {
                if log.message.to_lowercase().contains(&keyword.to_lowercase()) {
                    findings.push(AuditFinding {
                        id: generate_finding_id(),
                        title: format!("Performance Issue: {}", keyword),
                        description: format!("Performance keyword '{}' found in log: {}", keyword, log.message),
                        severity: match *keyword {
                            "timeout" | "overload" | "leak" => FindingSeverity::High,
                            "slow" | "latency" | "bottleneck" | "cpu" | "memory" => FindingSeverity::Medium,
                            _ => FindingSeverity::Low,
                        },
                        timestamp: log.timestamp,
                        related_entity: log.source.clone(),
                        location: "Log".to_string(),
                        recommendation: Some(format!("Investigate the {} issue in {}", keyword, log.source)),
                        metadata: log.metadata.clone(),
                    });
                    break; // Only create one finding per log entry
                }
            }
        }
        
        findings
    }
    
    fn audit_type(&self) -> AuditType {
        AuditType::Performance
    }
}

/// Compliance log analyzer for identifying compliance issues
struct ComplianceLogAnalyzer {}

impl ComplianceLogAnalyzer {
    /// Create a new compliance log analyzer
    fn new() -> Self {
        Self {}
    }
}

impl LogAnalyzer for ComplianceLogAnalyzer {
    fn analyze(&self, logs: &[LogEntry], context: &AuditContext) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        
        // Look for compliance-related keywords in logs
        let compliance_keywords = [
            "policy", "compliance", "violation", "regulation", "standard",
            "rule", "guideline", "protocol", "procedure", "requirement"
        ];
        
        for log in logs {
            for keyword in &compliance_keywords {
                if log.message.to_lowercase().contains(&keyword.to_lowercase()) {
                    findings.push(AuditFinding {
                        id: generate_finding_id(),
                        title: format!("Compliance Issue: {}", keyword),
                        description: format!("Compliance keyword '{}' found in log: {}", keyword, log.message),
                        severity: match *keyword {
                            "violation" => FindingSeverity::High,
                            "policy" | "compliance" | "regulation" | "standard" | "requirement" => FindingSeverity::Medium,
                            _ => FindingSeverity::Low,
                        },
                        timestamp: log.timestamp,
                        related_entity: log.source.clone(),
                        location: "Log".to_string(),
                        recommendation: Some(format!("Review {} compliance in {}", keyword, log.source)),
                        metadata: log.metadata.clone(),
                    });
                    break; // Only create one finding per log entry
                }
            }
        }
        
        findings
    }
    
    fn audit_type(&self) -> AuditType {
        AuditType::Compliance
    }
}

/// Agent behavior analyzer for monitoring agent operations
struct AgentBehaviorAnalyzer {}

impl AgentBehaviorAnalyzer {
    /// Create a new agent behavior analyzer
    fn new() -> Self {
        Self {}
    }
}

impl LogAnalyzer for AgentBehaviorAnalyzer {
    fn analyze(&self, logs: &[LogEntry], context: &AuditContext) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        
        // Look for agent behavior related keywords in logs
        let behavior_keywords = [
            "unexpected", "abnormal", "failure", "rejected", "crashed",
            "hanging", "unresponsive", "deadlock", "conflict", "stopped",
            "inconsistent", "invalid", "exception", "error"
        ];
        
        for log in logs {
            for keyword in &behavior_keywords {
                if log.message.to_lowercase().contains(&keyword.to_lowercase()) {
                    findings.push(AuditFinding {
                        id: generate_finding_id(),
                        title: format!("Agent Behavior Issue: {}", keyword),
                        description: format!("Agent behavior keyword '{}' found in log: {}", keyword, log.message),
                        severity: match *keyword {
                            "crashed" | "deadlock" | "unresponsive" => FindingSeverity::High,
                            "unexpected" | "abnormal" | "failure" | "hanging" | "inconsistent" => FindingSeverity::Medium,
                            _ => FindingSeverity::Low,
                        },
                        timestamp: log.timestamp,
                        related_entity: log.source.clone(),
                        location: "Log".to_string(),
                        recommendation: Some(format!("Investigate agent {} in {}", keyword, log.source)),
                        metadata: log.metadata.clone(),
                    });
                    break; // Only create one finding per log entry
                }
            }
        }
        
        findings
    }
    
    fn audit_type(&self) -> AuditType {
        AuditType::AgentBehavior
    }
}

/// Generate a unique ID for an audit report
fn generate_report_id() -> String {
    format!("report_{}", chrono::Utc::now().timestamp_nanos())
}

/// Generate a unique ID for an audit finding
fn generate_finding_id() -> String {
    format!("finding_{}", chrono::Utc::now().timestamp_nanos())
}

/// Generate a unique ID for an audit recommendation
fn generate_recommendation_id() -> String {
    format!("rec_{}", chrono::Utc::now().timestamp_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_security_analyzer() {
        let analyzer = SecurityLogAnalyzer::new();
        let now = Utc::now();
        
        let logs = vec![
            LogEntry {
                timestamp: now,
                source: "test-agent".to_string(),
                level: "ERROR".to_string(),
                message: "Unauthorized access attempt detected".to_string(),
                metadata: None,
            },
            LogEntry {
                timestamp: now,
                source: "test-service".to_string(),
                level: "WARN".to_string(),
                message: "Suspicious activity in login service".to_string(),
                metadata: None,
            },
        ];
        
        let context = AuditContext {
            start_time: Some(now - chrono::Duration::hours(1)),
            end_time: Some(now),
            target: "test-system".to_string(),
            parameters: HashMap::new(),
        };
        
        let findings = analyzer.analyze(&logs, &context);
        
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, FindingSeverity::High);
    }
    
    #[test]
    fn test_audit_service_report_generation() {
        let service = AuditService::new();
        let now = Utc::now();
        
        let logs = vec![
            LogEntry {
                timestamp: now,
                source: "test-agent".to_string(),
                level: "ERROR".to_string(),
                message: "Unauthorized access attempt detected".to_string(),
                metadata: None,
            },
            LogEntry {
                timestamp: now,
                source: "test-service".to_string(),
                level: "WARN".to_string(),
                message: "High CPU usage detected".to_string(),
                metadata: None,
            },
        ];
        
        let report = service.generate_report(
            AuditType::Security,
            "test-system".to_string(),
            &logs,
            Some(now - chrono::Duration::hours(1)),
            Some(now),
        );
        
        assert_eq!(report.audit_type, AuditType::Security);
        assert!(!report.findings.is_empty());
        assert!(!report.recommendations.is_empty());
        assert!(report.risk_score.is_some());
    }
}