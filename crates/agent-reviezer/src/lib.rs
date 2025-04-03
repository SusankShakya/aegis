mod audit;

use aegis_agent_framework::{
    agent::{AegisAgent, AgentContext, AgentError, AgentResult},
    comms::CommsClient,
    config::Config,
    platform::{AsyncTaskSpawner, AsyncTimer},
};
use audit::{
    LogAnalyzer, AgentLogEntry, AuditRecord, AuditService, AuditType,
    LogLevel, AuditContext,
};
use std::sync::Arc;
use chrono::Utc;
use std::collections::HashMap;

/// ReviezerAgent is responsible for auditing and log analysis within the Aegis system.
pub struct ReviezerAgent {
    context: AgentContext,
    comms: Arc<CommsClient>,
    audit_service: AuditService,
}

impl ReviezerAgent {
    pub fn new(context: AgentContext, comms: Arc<CommsClient>) -> Self {
        Self {
            context,
            comms,
            audit_service: AuditService::new(),
        }
    }

    async fn request_logs_from_agents(&self) -> AgentResult<Vec<AgentLogEntry>> {
        // TODO: Implement log request logic using comms
        // This is a placeholder for the actual implementation that will:
        // 1. Request logs from other agents via comms
        // 2. Convert received logs into AgentLogEntry format
        Ok(Vec::new())
    }

    async fn process_received_logs(&self, logs: Vec<AgentLogEntry>) -> AgentResult<AuditRecord> {
        // Create audit context
        let now = Utc::now();
        let context = AuditContext {
            start_time: Some(now - chrono::Duration::hours(1)),
            end_time: Some(now),
            target: self.context.agent_id().to_string(),
            parameters: HashMap::new(),
        };

        // Generate comprehensive audit report using the audit service
        let report = self.audit_service.generate_report(
            AuditType::AgentBehavior,
            self.context.agent_id().to_string(),
            &logs.iter().map(|log| LogEntry {
                timestamp: now,
                source: log.agent_id.clone(),
                level: match log.level {
                    LogLevel::Error => "ERROR".to_string(),
                    LogLevel::Warning => "WARN".to_string(),
                    LogLevel::Info => "INFO".to_string(),
                    LogLevel::Debug => "DEBUG".to_string(),
                },
                message: log.message.clone(),
                metadata: log.metadata.clone(),
            }).collect::<Vec<_>>(),
            Some(now - chrono::Duration::hours(1)),
            Some(now),
        );

        // Convert findings to AuditRecord format
        Ok(AuditRecord {
            timestamp: std::time::SystemTime::now(),
            agent_id: self.context.agent_id().to_string(),
            findings: report.findings.into_iter().map(|f| audit::AuditFinding {
                severity: match f.severity {
                    FindingSeverity::Critical => audit::AuditSeverity::Critical,
                    FindingSeverity::High => audit::AuditSeverity::High,
                    FindingSeverity::Medium => audit::AuditSeverity::Medium,
                    FindingSeverity::Low => audit::AuditSeverity::Low,
                    FindingSeverity::Info => audit::AuditSeverity::Info,
                },
                description: f.description,
                related_logs: logs.clone(), // For now, include all logs
            }).collect(),
        })
    }

    async fn store_audit_record(&self, record: AuditRecord) -> AgentResult<()> {
        // TODO: Implement storage logic for audit records
        // This could involve:
        // 1. Sending to a persistent store
        // 2. Notifying other agents of findings
        // 3. Triggering alerts for critical findings
        if record.findings.iter().any(|f| matches!(f.severity, audit::AuditSeverity::Critical)) {
            log::warn!("Critical findings detected in audit record");
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl AegisAgent for ReviezerAgent {
    fn agent_type(&self) -> &'static str {
        "reviezer"
    }

    fn context(&self) -> &AgentContext {
        &self.context
    }

    async fn run(&self) -> AgentResult<()> {
        loop {
            // Request logs from other agents
            match self.request_logs_from_agents().await {
                Ok(logs) => {
                    // Process logs and generate audit record
                    match self.process_received_logs(logs).await {
                        Ok(audit_record) => {
                            // Store or distribute the audit record
                            if let Err(e) = self.store_audit_record(audit_record).await {
                                log::error!("Failed to store audit record: {}", e);
                            }
                        }
                        Err(e) => log::error!("Error processing logs: {}", e),
                    }
                }
                Err(e) => log::error!("Error requesting logs: {}", e),
            }

            // Sleep for a configured interval before next cycle
            self.context.timer().sleep(std::time::Duration::from_secs(60)).await;
        }
    }

    async fn handle_shutdown(&self) -> AgentResult<()> {
        // Cleanup any resources if needed
        Ok(())
    }
}

// Required for the audit service
#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: chrono::DateTime<Utc>,
    source: String,
    level: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_agent_framework::test_utils::create_test_context;

    #[tokio::test]
    async fn test_reviezer_agent_creation() {
        let context = create_test_context("test_reviezer").await;
        let comms = Arc::new(CommsClient::default()); // Mock comms client for testing
        let agent = ReviezerAgent::new(context, comms);
        assert_eq!(agent.agent_type(), "reviezer");
    }

    #[tokio::test]
    async fn test_reviezer_agent_process_logs() {
        let context = create_test_context("test_reviezer").await;
        let comms = Arc::new(CommsClient::default());
        let agent = ReviezerAgent::new(context, comms);
        
        let logs = vec![AgentLogEntry {
            timestamp: std::time::SystemTime::now(),
            agent_id: "test_agent".to_string(),
            level: LogLevel::Error,
            message: "Test error".to_string(),
            metadata: None,
        }];

        let result = agent.process_received_logs(logs).await;
        assert!(result.is_ok());
        let audit_record = result.unwrap();
        assert!(!audit_record.findings.is_empty());
    }
} 