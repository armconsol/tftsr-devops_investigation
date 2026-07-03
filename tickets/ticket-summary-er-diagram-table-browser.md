# Ticket Summary

## Description
Fix the Database UI so the ER Diagram page no longer crashes on load and table data is visible through the GUI without requiring ad hoc SQL.

## Acceptance Criteria
- ER Diagram renders without the React Flow zustand-provider error.
- Table data can be browsed from a dedicated Database UI path.
- Existing database browsing behaviour continues to work.
- Frontend type-check, lint, and unit tests pass.

## Work Implemented
- Removed the `useReactFlow()` dependency from ER Diagram and fixed export to target the React Flow viewport with stable output sizing.
- Added a dedicated Table Browser page and wired it into the Database nav and routes.
- Added regression tests for the ER Diagram and Table Browser flows.
- Added a jsdom `ResizeObserver` test shim so React Flow mounts cleanly in unit tests.

## Testing Needed
- Manual smoke test of `/database/er-diagram` with a real database connection.
- Manual smoke test of `/database/browser` to confirm database/table selection and row browsing.
