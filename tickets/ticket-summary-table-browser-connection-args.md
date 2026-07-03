# Description

The table browser was failing to load rows because the frontend sent `connection_id` to Tauri commands that expect camelCase invoke keys.

# Acceptance Criteria

- Table browser commands send `connectionId` and related camelCase keys to Tauri.
- Loading table rows works without the missing `connectionId` error.
- Unit tests cover the table-browser command payloads.

# Work Implemented

- Normalised the table-browser invoke payloads in `src/lib/tauriCommands.ts`.
- Updated `TableBrowser` to pass `connectionId` into `browseTableDataCmd`.
- Added focused unit tests for browse, metadata, row count, and CRUD payloads.
- Updated the database wiki note for the table browser command contract.

# Testing Needed

- `npm run test:run -- tests/unit/TableBrowser.test.tsx tests/unit/TableBrowserPage.test.tsx tests/unit/tableBrowserCommands.test.ts`
- `npx tsc --noEmit`
- `npx eslint src/lib/tauriCommands.ts src/components/Database/TableBrowser.tsx tests/unit/tableBrowserCommands.test.ts --quiet`
