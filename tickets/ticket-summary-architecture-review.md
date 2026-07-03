# Architecture docs refresh

## Description
Refresh the `docs/architecture/` documentation so it matches the current v3.0.x application architecture, current modules, and implemented workflows.

## Acceptance Criteria
- `docs/architecture/README.md` reflects the current backend, frontend, and integration layout.
- Architecture diagrams and component lists match the shipped code.
- Affected ADRs no longer describe stale future work or outdated assumptions.
- The architecture docs remain consistent with the current wiki and app surface.

## Work Implemented
- Updated the main architecture README to reflect current modules, routes, stores, and system boundaries.
- Updated the MCP and Kubernetes ADRs to remove stale references and align with implemented behaviour.
- Corrected outdated counts and labels in architecture diagrams.

## Testing Needed
- Manual review of the rendered Markdown and Mermaid diagrams.
- Follow-up scan for stale references in `docs/architecture/` after any future architecture changes.
