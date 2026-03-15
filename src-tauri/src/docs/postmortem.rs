use crate::db::models::IssueDetail;

pub fn generate_postmortem_markdown(detail: &IssueDetail) -> String {
    let issue = &detail.issue;

    let mut md = String::new();

    md.push_str(&format!("# Blameless Post-Mortem: {}\n\n", issue.title));

    // Header metadata
    md.push_str("## Metadata\n\n");
    md.push_str(&format!("- **Date:** {}\n", issue.created_at));
    md.push_str(&format!("- **Severity:** {}\n", issue.severity));
    md.push_str(&format!("- **Category:** {}\n", issue.category));
    md.push_str(&format!("- **Status:** {}\n", issue.status));
    md.push_str(&format!("- **Last Updated:** {}\n", issue.updated_at));
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
    md.push_str("- **Duration:** _[How long did the incident last?]_\n");
    md.push_str("- **Users Affected:** _[Number/percentage of affected users]_\n");
    md.push_str("- **Revenue Impact:** _[Financial impact, if applicable]_\n");
    md.push_str("- **SLA Impact:** _[Were any SLAs breached?]_\n\n");

    // Timeline
    md.push_str("## Timeline\n\n");
    md.push_str("| Time (UTC) | Event |\n");
    md.push_str("|------------|-------|\n");
    md.push_str(&format!("| {} | Issue created |\n", issue.created_at));
    if let Some(ref resolved) = issue.resolved_at {
        md.push_str(&format!("| {resolved} | Issue resolved |\n"));
    }
    md.push_str("| _HH:MM_ | _[Add additional timeline events]_ |\n\n");

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
                "{}. **Why?** {} -> {}\n",
                step.step_order, step.why_question, answer
            ));
        }
        md.push('\n');

        if let Some(last) = detail.resolution_steps.last() {
            if !last.answer.is_empty() {
                md.push_str(&format!("**Root Cause:** {}\n\n", last.answer));
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
        "_Generated by TFTSR IT Triage on {}_\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Issue, IssueDetail, ResolutionStep};

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
}
