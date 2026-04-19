use crate::db::models::IssueDetail;

pub fn format_event_type(event_type: &str) -> &str {
    match event_type {
        "triage_started" => "Triage Started",
        "log_uploaded" => "Log File Uploaded",
        "why_level_advanced" => "Why Level Advanced",
        "root_cause_identified" => "Root Cause Identified",
        "rca_generated" => "RCA Document Generated",
        "postmortem_generated" => "Post-Mortem Generated",
        "document_exported" => "Document Exported",
        other => other,
    }
}

pub fn calculate_duration(start: &str, end: &str) -> String {
    let fmt = "%Y-%m-%d %H:%M:%S UTC";
    let start_dt = match chrono::NaiveDateTime::parse_from_str(start, fmt) {
        Ok(dt) => dt,
        Err(_) => return "N/A".to_string(),
    };
    let end_dt = match chrono::NaiveDateTime::parse_from_str(end, fmt) {
        Ok(dt) => dt,
        Err(_) => return "N/A".to_string(),
    };

    let duration = end_dt.signed_duration_since(start_dt);
    let total_minutes = duration.num_minutes();
    if total_minutes < 0 {
        return "N/A".to_string();
    }

    let days = total_minutes / (24 * 60);
    let hours = (total_minutes % (24 * 60)) / 60;
    let minutes = total_minutes % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

pub fn generate_rca_markdown(detail: &IssueDetail) -> String {
    let issue = &detail.issue;

    let mut md = String::new();

    md.push_str(&format!(
        "# Root Cause Analysis: {title}\n\n",
        title = issue.title
    ));

    md.push_str("## Issue Summary\n\n");
    md.push_str("| Field | Value |\n");
    md.push_str("|-------|-------|\n");
    md.push_str(&format!("| **Issue ID** | {id} |\n", id = issue.id));
    md.push_str(&format!(
        "| **Category** | {category} |\n",
        category = issue.category
    ));
    md.push_str(&format!(
        "| **Status** | {status} |\n",
        status = issue.status
    ));
    md.push_str(&format!(
        "| **Severity** | {severity} |\n",
        severity = issue.severity
    ));
    md.push_str(&format!(
        "| **Source** | {source} |\n",
        source = issue.source
    ));
    md.push_str(&format!(
        "| **Assigned To** | {} |\n",
        if issue.assigned_to.is_empty() {
            "Unassigned"
        } else {
            &issue.assigned_to
        }
    ));
    md.push_str(&format!(
        "| **Created** | {created_at} |\n",
        created_at = issue.created_at
    ));
    md.push_str(&format!(
        "| **Last Updated** | {updated_at} |\n",
        updated_at = issue.updated_at
    ));
    if let Some(ref resolved) = issue.resolved_at {
        md.push_str(&format!("| **Resolved** | {resolved} |\n"));
    }
    md.push('\n');

    if !issue.description.is_empty() {
        md.push_str("## Description\n\n");
        md.push_str(&issue.description);
        md.push_str("\n\n");
    }

    // Incident Timeline
    md.push_str("## Incident Timeline\n\n");
    if detail.timeline_events.is_empty() {
        md.push_str("_No timeline events recorded._\n\n");
    } else {
        md.push_str("| Time (UTC) | Event | Description |\n");
        md.push_str("|------------|-------|-------------|\n");
        for event in &detail.timeline_events {
            md.push_str(&format!(
                "| {} | {} | {} |\n",
                event.created_at,
                format_event_type(&event.event_type),
                event.description
            ));
        }
        md.push('\n');
    }

    // Incident Metrics
    md.push_str("## Incident Metrics\n\n");
    md.push_str(&format!(
        "- **Total Events:** {}\n",
        detail.timeline_events.len()
    ));
    if detail.timeline_events.len() >= 2 {
        let first = &detail.timeline_events[0].created_at;
        let last = &detail.timeline_events[detail.timeline_events.len() - 1].created_at;
        md.push_str(&format!(
            "- **Incident Duration:** {}\n",
            calculate_duration(first, last)
        ));
    } else {
        md.push_str("- **Incident Duration:** N/A\n");
    }
    let root_cause_event = detail
        .timeline_events
        .iter()
        .find(|e| e.event_type == "root_cause_identified");
    if let (Some(first), Some(rc)) = (detail.timeline_events.first(), root_cause_event) {
        md.push_str(&format!(
            "- **Time to Root Cause:** {}\n",
            calculate_duration(&first.created_at, &rc.created_at)
        ));
    }
    md.push('\n');

    // 5 Whys Analysis
    md.push_str("## 5 Whys Analysis\n\n");
    if detail.resolution_steps.is_empty() {
        md.push_str("_No 5-whys analysis has been performed yet._\n\n");
    } else {
        for step in &detail.resolution_steps {
            md.push_str(&format!(
                "### Why #{}: {}\n\n",
                step.step_order, step.why_question
            ));
            if !step.answer.is_empty() {
                md.push_str(&format!("**Answer:** {answer}\n\n", answer = step.answer));
            } else {
                md.push_str("_Awaiting answer._\n\n");
            }
            if !step.evidence.is_empty() {
                md.push_str(&format!(
                    "**Evidence:** {evidence}\n\n",
                    evidence = step.evidence
                ));
            }
        }
    }

    // Root Cause
    md.push_str("## Root Cause\n\n");
    if let Some(last_step) = detail.resolution_steps.last() {
        if !last_step.answer.is_empty() {
            md.push_str(&format!(
                "Based on the 5-whys analysis, the root cause is:\n\n> {}\n\n",
                last_step.answer
            ));
        } else {
            md.push_str(
                "_The 5-whys analysis is incomplete. Complete it to identify the root cause._\n\n",
            );
        }
    } else {
        md.push_str("_Perform the 5-whys analysis to identify the root cause._\n\n");
    }

    // Log Files
    md.push_str("## Log Files Analyzed\n\n");
    if detail.log_files.is_empty() {
        md.push_str("_No log files attached._\n\n");
    } else {
        md.push_str("| File | Size | Redacted | Hash |\n");
        md.push_str("|------|------|----------|------|\n");
        for lf in &detail.log_files {
            md.push_str(&format!(
                "| {} | {} bytes | {} | {}... |\n",
                lf.file_name,
                lf.file_size,
                if lf.redacted { "Yes" } else { "No" },
                &lf.content_hash[..8.min(lf.content_hash.len())],
            ));
        }
        md.push('\n');
    }

    // Corrective Actions
    md.push_str("## Corrective Actions\n\n");
    md.push_str("### Immediate Actions\n\n");
    md.push_str("- [ ] _Document immediate mitigations taken_\n\n");
    md.push_str("### Long-Term Actions\n\n");
    md.push_str("- [ ] _Document preventive measures_\n");
    md.push_str("- [ ] _Document monitoring improvements_\n\n");

    // Lessons Learned
    md.push_str("## Lessons Learned\n\n");
    md.push_str("- _What went well during the response?_\n");
    md.push_str("- _What could be improved?_\n");
    md.push_str("- _What processes need updating?_\n\n");

    md.push_str("---\n\n");
    md.push_str(&format!(
        "_Generated by Troubleshooting and RCA Assistant on {}_\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Issue, IssueDetail, LogFile, ResolutionStep, TimelineEvent};

    fn make_test_detail() -> IssueDetail {
        IssueDetail {
            issue: Issue {
                id: "test-123".to_string(),
                title: "Database connection timeout".to_string(),
                description: "Users report 500 errors on login.".to_string(),
                severity: "high".to_string(),
                status: "investigating".to_string(),
                category: "database".to_string(),
                source: "manual".to_string(),
                created_at: "2025-01-15 10:00:00".to_string(),
                updated_at: "2025-01-15 12:00:00".to_string(),
                resolved_at: None,
                assigned_to: "oncall-eng".to_string(),
                tags: "[]".to_string(),
            },
            log_files: vec![LogFile {
                id: "lf-1".to_string(),
                issue_id: "test-123".to_string(),
                file_name: "app.log".to_string(),
                file_path: "/tmp/app.log".to_string(),
                file_size: 2048,
                mime_type: "text/plain".to_string(),
                content_hash: "abc123def456".to_string(),
                uploaded_at: "2025-01-15 10:30:00".to_string(),
                redacted: false,
            }],
            image_attachments: vec![],
            resolution_steps: vec![
                ResolutionStep {
                    id: "rs-1".to_string(),
                    issue_id: "test-123".to_string(),
                    step_order: 1,
                    why_question: "Why are users getting 500 errors?".to_string(),
                    answer: "The database connection pool is exhausted.".to_string(),
                    evidence: "Connection pool metrics show 100/100 used.".to_string(),
                    created_at: "2025-01-15 11:00:00".to_string(),
                },
                ResolutionStep {
                    id: "rs-2".to_string(),
                    issue_id: "test-123".to_string(),
                    step_order: 2,
                    why_question: "Why is the connection pool exhausted?".to_string(),
                    answer: "Queries are not being released after completion.".to_string(),
                    evidence: "".to_string(),
                    created_at: "2025-01-15 11:15:00".to_string(),
                },
            ],
            conversations: vec![],
            timeline_events: vec![],
        }
    }

    #[test]
    fn test_rca_contains_title() {
        let md = generate_rca_markdown(&make_test_detail());
        assert!(md.contains("# Root Cause Analysis: Database connection timeout"));
    }

    #[test]
    fn test_rca_contains_issue_summary_table() {
        let md = generate_rca_markdown(&make_test_detail());
        assert!(md.contains("| **Issue ID** | test-123 |"));
        assert!(md.contains("| **Severity** | high |"));
        assert!(md.contains("| **Assigned To** | oncall-eng |"));
    }

    #[test]
    fn test_rca_contains_five_whys() {
        let md = generate_rca_markdown(&make_test_detail());
        assert!(md.contains("### Why #1: Why are users getting 500 errors?"));
        assert!(md.contains("**Answer:** The database connection pool is exhausted."));
        assert!(md.contains("**Evidence:** Connection pool metrics"));
    }

    #[test]
    fn test_rca_contains_root_cause() {
        let md = generate_rca_markdown(&make_test_detail());
        assert!(md.contains("Queries are not being released after completion."));
    }

    #[test]
    fn test_rca_contains_log_files() {
        let md = generate_rca_markdown(&make_test_detail());
        assert!(md.contains("app.log"));
        assert!(md.contains("2048 bytes"));
    }

    #[test]
    fn test_rca_empty_steps_shows_placeholder() {
        let mut detail = make_test_detail();
        detail.resolution_steps.clear();
        let md = generate_rca_markdown(&detail);
        assert!(md.contains("No 5-whys analysis has been performed yet."));
    }

    #[test]
    fn test_rca_unassigned_shows_unassigned() {
        let mut detail = make_test_detail();
        detail.issue.assigned_to = String::new();
        let md = generate_rca_markdown(&detail);
        assert!(md.contains("Unassigned"));
    }

    #[test]
    fn test_rca_timeline_section_with_events() {
        let mut detail = make_test_detail();
        detail.timeline_events = vec![
            TimelineEvent {
                id: "te-1".to_string(),
                issue_id: "test-123".to_string(),
                event_type: "triage_started".to_string(),
                description: "Triage initiated by oncall".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-01-15 10:00:00 UTC".to_string(),
            },
            TimelineEvent {
                id: "te-2".to_string(),
                issue_id: "test-123".to_string(),
                event_type: "log_uploaded".to_string(),
                description: "app.log uploaded".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-01-15 10:30:00 UTC".to_string(),
            },
            TimelineEvent {
                id: "te-3".to_string(),
                issue_id: "test-123".to_string(),
                event_type: "root_cause_identified".to_string(),
                description: "Connection pool leak found".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-01-15 12:15:00 UTC".to_string(),
            },
        ];
        let md = generate_rca_markdown(&detail);
        assert!(md.contains("## Incident Timeline"));
        assert!(md.contains("| Time (UTC) | Event | Description |"));
        assert!(md
            .contains("| 2025-01-15 10:00:00 UTC | Triage Started | Triage initiated by oncall |"));
        assert!(md.contains("| 2025-01-15 10:30:00 UTC | Log File Uploaded | app.log uploaded |"));
        assert!(md.contains(
            "| 2025-01-15 12:15:00 UTC | Root Cause Identified | Connection pool leak found |"
        ));
    }

    #[test]
    fn test_rca_timeline_section_empty() {
        let detail = make_test_detail();
        let md = generate_rca_markdown(&detail);
        assert!(md.contains("## Incident Timeline"));
        assert!(md.contains("_No timeline events recorded._"));
    }

    #[test]
    fn test_rca_metrics_section() {
        let mut detail = make_test_detail();
        detail.timeline_events = vec![
            TimelineEvent {
                id: "te-1".to_string(),
                issue_id: "test-123".to_string(),
                event_type: "triage_started".to_string(),
                description: "Triage started".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-01-15 10:00:00 UTC".to_string(),
            },
            TimelineEvent {
                id: "te-2".to_string(),
                issue_id: "test-123".to_string(),
                event_type: "root_cause_identified".to_string(),
                description: "Root cause found".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-01-15 12:15:00 UTC".to_string(),
            },
        ];
        let md = generate_rca_markdown(&detail);
        assert!(md.contains("## Incident Metrics"));
        assert!(md.contains("**Total Events:** 2"));
        assert!(md.contains("**Incident Duration:** 2h 15m"));
        assert!(md.contains("**Time to Root Cause:** 2h 15m"));
    }

    #[test]
    fn test_calculate_duration_hours_minutes() {
        assert_eq!(
            calculate_duration("2025-01-15 10:00:00 UTC", "2025-01-15 12:15:00 UTC"),
            "2h 15m"
        );
    }

    #[test]
    fn test_calculate_duration_days() {
        assert_eq!(
            calculate_duration("2025-01-15 10:00:00 UTC", "2025-01-18 11:00:00 UTC"),
            "3d 1h"
        );
    }

    #[test]
    fn test_calculate_duration_minutes_only() {
        assert_eq!(
            calculate_duration("2025-01-15 10:00:00 UTC", "2025-01-15 10:45:00 UTC"),
            "45m"
        );
    }

    #[test]
    fn test_calculate_duration_invalid() {
        assert_eq!(calculate_duration("bad-date", "also-bad"), "N/A");
    }

    #[test]
    fn test_format_event_type_known() {
        assert_eq!(format_event_type("triage_started"), "Triage Started");
        assert_eq!(format_event_type("log_uploaded"), "Log File Uploaded");
        assert_eq!(
            format_event_type("why_level_advanced"),
            "Why Level Advanced"
        );
        assert_eq!(
            format_event_type("root_cause_identified"),
            "Root Cause Identified"
        );
        assert_eq!(format_event_type("rca_generated"), "RCA Document Generated");
        assert_eq!(
            format_event_type("postmortem_generated"),
            "Post-Mortem Generated"
        );
        assert_eq!(format_event_type("document_exported"), "Document Exported");
    }

    #[test]
    fn test_format_event_type_unknown() {
        assert_eq!(format_event_type("custom_event"), "custom_event");
        assert_eq!(format_event_type(""), "");
    }
}
