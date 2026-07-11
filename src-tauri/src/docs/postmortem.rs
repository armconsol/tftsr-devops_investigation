use crate::db::models::IssueDetail;
use crate::docs::rca::{calculate_duration, format_event_type};

pub fn generate_postmortem_markdown(detail: &IssueDetail) -> String {
    let issue = &detail.issue;

    let mut md = String::new();

    md.push_str(&format!(
        "# Blameless Post-Mortem: {title}\n\n",
        title = issue.title
    ));

    // Header metadata
    md.push_str("## Metadata\n\n");
    md.push_str(&format!(
        "- **Date:** {created_at}\n",
        created_at = issue.created_at
    ));
    md.push_str(&format!(
        "- **Severity:** {severity}\n",
        severity = issue.severity
    ));
    md.push_str(&format!(
        "- **Category:** {category}\n",
        category = issue.category
    ));
    md.push_str(&format!("- **Status:** {status}\n", status = issue.status));
    md.push_str(&format!(
        "- **Last Updated:** {updated_at}\n",
        updated_at = issue.updated_at
    ));
    md.push_str(&format!(
        "- **Assigned To:** {}\n",
        if issue.assigned_to.is_empty() {
            "_Unassigned_"
        } else {
            &issue.assigned_to
        }
    ));
    md.push_str("- **Authors:** _[Add authors]_\n");
    md.push_str("- **Reviewers:** _[Add reviewers]_\n\n");

    // Executive Summary
    md.push_str("## Executive Summary\n\n");
    if !issue.description.is_empty() {
        md.push_str(&issue.description);
        md.push_str("\n\n");
    } else {
        md.push_str("_Provide a brief executive summary of the incident._\n\n");
    }

    // Impact
    md.push_str("## Impact\n\n");
    if detail.timeline_events.len() >= 2 {
        let first = &detail.timeline_events[0].created_at;
        let last = &detail.timeline_events[detail.timeline_events.len() - 1].created_at;
        md.push_str(&format!(
            "- **Duration:** {}\n",
            calculate_duration(first, last)
        ));
    } else {
        md.push_str("- **Duration:** _[How long did the incident last?]_\n");
    }
    md.push_str("- **Users Affected:** _[Number/percentage of affected users]_\n");
    md.push_str("- **Revenue Impact:** _[Financial impact, if applicable]_\n");
    md.push_str("- **SLA Impact:** _[Were any SLAs breached?]_\n\n");

    // Timeline
    md.push_str("## Timeline\n\n");
    md.push_str("| Time (UTC) | Event |\n");
    md.push_str("|------------|-------|\n");
    md.push_str(&format!(
        "| {created_at} | Issue created |\n",
        created_at = issue.created_at
    ));
    if let Some(ref resolved) = issue.resolved_at {
        md.push_str(&format!("| {resolved} | Issue resolved |\n"));
    }
    if detail.timeline_events.is_empty() {
        md.push_str("| _HH:MM_ | _[Add additional timeline events]_ |\n");
    } else {
        for event in &detail.timeline_events {
            md.push_str(&format!(
                "| {} | {} - {} |\n",
                event.created_at,
                format_event_type(&event.event_type),
                event.description
            ));
        }
    }
    md.push('\n');

    // Root Cause Analysis
    md.push_str("## Root Cause Analysis\n\n");
    if detail.resolution_steps.is_empty() {
        md.push_str("### 5 Whys\n\n");
        md.push_str("1. **Why?** _[First question]_ -> _[Answer]_\n");
        md.push_str("2. **Why?** _[Second question]_ -> _[Answer]_\n");
        md.push_str("3. **Why?** _[Third question]_ -> _[Answer]_\n");
        md.push_str("4. **Why?** _[Fourth question]_ -> _[Answer]_\n");
        md.push_str("5. **Why?** _[Fifth question]_ -> _[Answer]_\n\n");
    } else {
        md.push_str("### 5 Whys\n\n");
        for step in &detail.resolution_steps {
            let answer = if step.answer.is_empty() {
                "_Awaiting answer_"
            } else {
                &step.answer
            };
            md.push_str(&format!(
                "{}. **Why?** {} -> {answer}\n",
                step.step_order, step.why_question
            ));
        }
        md.push('\n');

        if let Some(last) = detail.resolution_steps.last() {
            if !last.answer.is_empty() {
                md.push_str(&format!(
                    "**Root Cause:** {answer}\n\n",
                    answer = last.answer
                ));
            }
        }
    }

    // Contributing Factors
    md.push_str("## Contributing Factors\n\n");
    md.push_str(
        "_This is a blameless post-mortem. Focus on systems and processes, not individuals._\n\n",
    );
    md.push_str("- _[Factor 1: e.g., Insufficient monitoring coverage]_\n");
    md.push_str("- _[Factor 2: e.g., Missing automated failover]_\n");
    md.push_str("- _[Factor 3: e.g., Deployment timing during peak hours]_\n\n");

    // What Went Well
    md.push_str("## What Went Well\n\n");
    if !detail.resolution_steps.is_empty() {
        md.push_str(&format!(
            "- Systematic 5-whys analysis conducted ({} steps completed)\n",
            detail.resolution_steps.len()
        ));
    }
    if detail
        .timeline_events
        .iter()
        .any(|e| e.event_type == "root_cause_identified")
    {
        md.push_str("- Root cause was identified during triage\n");
    }
    md.push_str("- _[e.g., Quick detection through existing alerts]_\n");
    md.push_str("- _[e.g., Effective cross-team collaboration]_\n");
    md.push_str("- _[e.g., Smooth communication with stakeholders]_\n\n");

    // What Could Be Improved
    md.push_str("## What Could Be Improved\n\n");
    md.push_str("- _[e.g., Faster escalation path]_\n");
    md.push_str("- _[e.g., Better runbook documentation]_\n");
    md.push_str("- _[e.g., More comprehensive testing]_\n\n");

    // Action Items
    md.push_str("## Action Items\n\n");
    md.push_str("| Priority | Action | Owner | Due Date | Status |\n");
    md.push_str("|----------|--------|-------|----------|--------|\n");
    md.push_str("| P0 | _[Critical fix]_ | _[Owner]_ | _[Date]_ | Open |\n");
    md.push_str("| P1 | _[Prevention measure]_ | _[Owner]_ | _[Date]_ | Open |\n");
    md.push_str("| P2 | _[Monitoring improvement]_ | _[Owner]_ | _[Date]_ | Open |\n\n");

    // Log Files
    if !detail.log_files.is_empty() {
        md.push_str("## Appendix: Log Files\n\n");
        for lf in &detail.log_files {
            md.push_str(&format!(
                "- `{}` ({} bytes, redacted: {})\n",
                lf.file_name,
                lf.file_size,
                if lf.redacted { "yes" } else { "no" }
            ));
        }
        md.push('\n');
    }

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
    use crate::db::models::{Issue, IssueDetail, ResolutionStep, TimelineEvent};

    fn make_test_detail() -> IssueDetail {
        IssueDetail {
            issue: Issue {
                id: "pm-456".to_string(),
                title: "Payment processing outage".to_string(),
                description: "Payment gateway returning 503 for all transactions.".to_string(),
                severity: "critical".to_string(),
                status: "resolved".to_string(),
                category: "payments".to_string(),
                source: "monitoring".to_string(),
                created_at: "2025-02-10 08:00:00".to_string(),
                updated_at: "2025-02-10 14:00:00".to_string(),
                resolved_at: Some("2025-02-10 12:30:00".to_string()),
                assigned_to: "payments-team".to_string(),
                tags: "[]".to_string(),
            },
            log_files: vec![],
            image_attachments: vec![],
            resolution_steps: vec![ResolutionStep {
                id: "rs-pm-1".to_string(),
                issue_id: "pm-456".to_string(),
                step_order: 1,
                why_question: "Why did payments fail?".to_string(),
                answer: "Gateway certificate expired.".to_string(),
                evidence: "SSL handshake failure in logs.".to_string(),
                created_at: "2025-02-10 09:00:00".to_string(),
            }],
            conversations: vec![],
            timeline_events: vec![],
        }
    }

    #[test]
    fn test_postmortem_contains_blameless_title() {
        let md = generate_postmortem_markdown(&make_test_detail());
        assert!(md.contains("# Blameless Post-Mortem: Payment processing outage"));
    }

    #[test]
    fn test_postmortem_contains_metadata() {
        let md = generate_postmortem_markdown(&make_test_detail());
        assert!(md.contains("- **Severity:** critical"));
        assert!(md.contains("- **Category:** payments"));
        assert!(md.contains("- **Assigned To:** payments-team"));
    }

    #[test]
    fn test_postmortem_contains_executive_summary() {
        let md = generate_postmortem_markdown(&make_test_detail());
        assert!(md.contains("Payment gateway returning 503"));
    }

    #[test]
    fn test_postmortem_contains_timeline_with_resolved() {
        let md = generate_postmortem_markdown(&make_test_detail());
        assert!(md.contains("2025-02-10 08:00:00"));
        assert!(md.contains("2025-02-10 12:30:00"));
        assert!(md.contains("Issue resolved"));
    }

    #[test]
    fn test_postmortem_contains_five_whys() {
        let md = generate_postmortem_markdown(&make_test_detail());
        assert!(md.contains("Why did payments fail?"));
        assert!(md.contains("Gateway certificate expired."));
    }

    #[test]
    fn test_postmortem_contains_blameless_reminder() {
        let md = generate_postmortem_markdown(&make_test_detail());
        assert!(md.contains("blameless post-mortem"));
    }

    #[test]
    fn test_postmortem_empty_description_shows_placeholder() {
        let mut detail = make_test_detail();
        detail.issue.description = String::new();
        let md = generate_postmortem_markdown(&detail);
        assert!(md.contains("Provide a brief executive summary"));
    }

    #[test]
    fn test_postmortem_action_items_table() {
        let md = generate_postmortem_markdown(&make_test_detail());
        assert!(md.contains("| Priority | Action | Owner | Due Date | Status |"));
        assert!(md.contains("| P0 |"));
    }

    #[test]
    fn test_postmortem_timeline_with_real_events() {
        let mut detail = make_test_detail();
        detail.timeline_events = vec![
            TimelineEvent {
                id: "te-1".to_string(),
                issue_id: "pm-456".to_string(),
                event_type: "triage_started".to_string(),
                description: "Triage initiated".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-02-10 08:05:00 UTC".to_string(),
            },
            TimelineEvent {
                id: "te-2".to_string(),
                issue_id: "pm-456".to_string(),
                event_type: "root_cause_identified".to_string(),
                description: "Certificate expiry confirmed".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-02-10 10:30:00 UTC".to_string(),
            },
        ];
        let md = generate_postmortem_markdown(&detail);
        assert!(md.contains("## Timeline"));
        assert!(md.contains("| 2025-02-10 08:05:00 UTC | Triage Started - Triage initiated |"));
        assert!(md.contains(
            "| 2025-02-10 10:30:00 UTC | Root Cause Identified - Certificate expiry confirmed |"
        ));
        assert!(!md.contains("_[Add additional timeline events]_"));
    }

    #[test]
    fn test_postmortem_impact_with_duration() {
        let mut detail = make_test_detail();
        detail.timeline_events = vec![
            TimelineEvent {
                id: "te-1".to_string(),
                issue_id: "pm-456".to_string(),
                event_type: "triage_started".to_string(),
                description: "Triage initiated".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-02-10 08:00:00 UTC".to_string(),
            },
            TimelineEvent {
                id: "te-2".to_string(),
                issue_id: "pm-456".to_string(),
                event_type: "root_cause_identified".to_string(),
                description: "Found it".to_string(),
                metadata: "{}".to_string(),
                created_at: "2025-02-10 10:30:00 UTC".to_string(),
            },
        ];
        let md = generate_postmortem_markdown(&detail);
        assert!(md.contains("**Duration:** 2h 30m"));
        assert!(!md.contains("_[How long did the incident last?]_"));
    }

    #[test]
    fn test_postmortem_what_went_well_with_steps() {
        let mut detail = make_test_detail();
        detail.timeline_events = vec![TimelineEvent {
            id: "te-1".to_string(),
            issue_id: "pm-456".to_string(),
            event_type: "root_cause_identified".to_string(),
            description: "Root cause found".to_string(),
            metadata: "{}".to_string(),
            created_at: "2025-02-10 10:00:00 UTC".to_string(),
        }];
        let md = generate_postmortem_markdown(&detail);
        assert!(md.contains("Systematic 5-whys analysis conducted (1 steps completed)"));
        assert!(md.contains("Root cause was identified during triage"));
    }
}
