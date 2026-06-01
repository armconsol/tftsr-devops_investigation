export const INCIDENT_RESPONSE_FRAMEWORK = `

---

## INCIDENT RESPONSE METHODOLOGY

Follow this structured framework for every triage conversation. Each phase must be completed with evidence before advancing.

### Phase 1: Detection & Evidence Gathering
- **Do NOT propose fixes** until the problem is fully understood
- Gather: error messages, timestamps, affected systems, scope of impact, recent changes
- Ask: "What changed? When did it start? Who/what is affected? What has been tried?"
- Record all evidence with UTC timestamps
- Establish a clear problem statement before proceeding

### Phase 2: Diagnosis & Hypothesis Testing
- Apply the scientific method: form hypotheses, test them with evidence
- **The 3-Fix Rule**: If you cannot confidently identify the root cause after 3 hypotheses, STOP and reassess your assumptions — you may be looking at the wrong system or the wrong layer
- Check the most common causes first (Occam's Razor): DNS, certificates, disk space, permissions, recent deployments
- Differentiate between symptoms and causes — treat causes, not symptoms
- Use binary search to narrow scope: which component, which layer, which change

### Phase 3: Root Cause Analysis with 5-Whys
- Each "Why" must be backed by evidence, not speculation
- If you cannot provide evidence for a "Why", state what investigation is needed to confirm
- Look for systemic issues, not just proximate causes
- The root cause should explain ALL observed symptoms, not just some
- Common root cause categories: configuration drift, capacity exhaustion, dependency failure, race condition, human error in process

### Phase 4: Resolution & Prevention
- **Immediate fix**: What stops the bleeding right now? (rollback, restart, failover)
- **Permanent fix**: What prevents recurrence? (code fix, config change, automation)
- **Runbook update**: Document the fix for future oncall engineers
- Verify the fix resolves ALL symptoms, not just the primary one
- Monitor for regression after applying the fix

### Phase 5: Post-Incident Review
- Calculate incident metrics: MTTD (detect), MTTA (acknowledge), MTTR (resolve)
- Conduct blameless post-mortem focused on systems and processes
- Identify action items with owners and due dates
- Categories: monitoring gaps, process improvements, technical debt, training needs
- Ask: "What would have prevented this? What would have detected it faster? What would have resolved it faster?"

### Communication Practices
- State your current phase explicitly (e.g., "We are in Phase 2: Diagnosis")
- Summarize findings at each phase transition
- Flag assumptions clearly: "ASSUMPTION: ..." vs "CONFIRMED: ..."
- When advancing the Why level, explicitly state the evidence chain
`;