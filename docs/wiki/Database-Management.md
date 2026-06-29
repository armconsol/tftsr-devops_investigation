# Database Management

> **Version:** 3.0.0  
> **Status:** Production Ready  
> **Last Updated:** 2025-06-29

## Overview

TRCAA v3.0.0 introduces comprehensive multi-database management capabilities, enabling users to connect, query, analyze, and visualize data across five major database systems within the incident response workflow.

### Supported Database Systems

| Database | Version Support | Driver | Transaction Support | Schema Introspection |
|----------|----------------|--------|---------------------|---------------------|
| **PostgreSQL** | 10.0+ | tokio-postgres | ✅ Full | ✅ Complete |
| **MySQL** | 5.7+, 8.0+ | mysql_async | ✅ Full | ✅ Complete |
| **MongoDB** | 4.0+ | mongodb (async) | ⚠️ Limited | ✅ Collection-based |
| **Redis** | 5.0+ | redis (async) | ⚠️ MULTI/EXEC | ⚠️ Key-pattern based |
| **Cassandra** | 3.11+, 4.0+ | scylla (CQL) | ❌ No | ✅ Keyspace-based |

### Key Features

- **Unified Connection Management** — Single interface for all database types with encrypted credential storage
- **Live Query Execution** — Execute native queries (SQL, CQL, MongoDB commands) with real-time results
- **Schema Visualization** — Auto-generate ER diagrams showing tables, columns, and relationships
- **Data Import/Export** — CSV, JSON, and SQL formats with streaming support for large datasets
- **Connection Pooling** — Persistent connections across sessions with automatic cleanup
- **Audit Logging** — Full audit trail of all database operations for compliance

---

## Getting Started

### Quick Start

1. **Navigate to Settings → Databases**
2. **Add New Connection** — Choose database type and enter credentials
3. **Test Connection** — Verify connectivity before saving
4. **Execute Query** — Run your first query from the Query tab
5. **Export Results** — Save query results to CSV/JSON/SQL

### Basic Connection Example

```typescript
// Example PostgreSQL connection
const config = {
  database_type: "PostgreSQL",
  host: "localhost",
  port: 5432,
  database: "production_db",
  username: "readonly_user",
  password: "secure_password",
  ssl_config: {
    enabled: true,
    verify_server: true
  }
};

// Test connection
await testDatabaseConnection(config);

// Execute query
const result = await executeQuery(
  connectionId,
  "SELECT * FROM incidents WHERE severity = 'Critical' ORDER BY created_at DESC LIMIT 10"
);
```

---

## Features

### 1. Connection Management

#### Create Connection

Connections are stored encrypted in the local SQLite database. Each connection requires:

- **Database Type** — PostgreSQL, MySQL, MongoDB, Redis, or Cassandra
- **Host/Port** — Connection endpoint (defaults to standard ports)
- **Credentials** — Username/password (encrypted with AES-256-GCM)
- **Database Name** — Target database/keyspace (optional for Redis)
- **SSL Configuration** — TLS settings with certificate paths

**Connection Storage:**

```rust
// Connections stored in `database_connections` table
CREATE TABLE database_connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    database_type TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    database TEXT,
    username TEXT NOT NULL,
    encrypted_password TEXT NOT NULL,  // AES-256-GCM encrypted
    ssl_enabled INTEGER DEFAULT 0,
    ssl_config TEXT,  // JSON
    options TEXT,     // JSON
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

#### Test Connection

Before saving, test connectivity and retrieve server version:

```typescript
const status = await testDatabaseConnection(config);
// Returns: { is_connected, message, server_version, latency_ms }
```

#### Connection Pooling

The `DatabasePoolManager` maintains persistent connections across sessions:

- **Automatic Creation** — Connections created on first use
- **Connection Reuse** — Same connection used for multiple queries
- **Cleanup on Disconnect** — Graceful shutdown when removing connections
- **Session Isolation** — Each connection ID maps to isolated pool

**Pool Manager Methods:**

```rust
// Get or create driver for connection
async fn get_or_create_driver(connection_id: &str, config: &ConnectionConfig)
    -> DriverResult<Arc<RwLock<Box<dyn DatabaseDriver>>>>;

// Remove driver from pool (disconnects)
async fn remove_driver(connection_id: &str) -> DriverResult<()>;

// Clear all connections
async fn clear_all() -> DriverResult<()>;

// Get active connection count
async fn active_count() -> usize;
```

---

### 2. Query Execution

#### Execute Native Queries

Each database supports its native query language:

**PostgreSQL/MySQL:**
```sql
-- SQL queries with parameters
SELECT user_id, COUNT(*) as incident_count
FROM incidents
WHERE created_at > $1
GROUP BY user_id
ORDER BY incident_count DESC;
```

**MongoDB:**
```javascript
// MongoDB aggregation pipeline (as JSON string)
db.incidents.aggregate([
  { $match: { severity: "Critical" } },
  { $group: { _id: "$assigned_to", count: { $sum: 1 } } },
  { $sort: { count: -1 } }
])
```

**Cassandra:**
```cql
-- CQL queries
SELECT * FROM incidents_by_date
WHERE partition_date = '2025-06-29'
AND severity = 'Critical'
ALLOW FILTERING;
```

**Redis:**
```bash
# Redis commands (parsed as strings)
HGETALL incident:12345
ZRANGE recent_incidents 0 9 WITHSCORES
```

#### Query Results

All queries return a unified `QueryResult` structure:

```typescript
interface QueryResult {
  columns: ColumnMetadata[];  // Column names and types
  rows: DataValue[][];        // 2D array of row data
  row_count: number;          // Total rows returned
  execution_time_ms: number;  // Query execution time
}

interface ColumnMetadata {
  name: string;
  data_type: string;          // Database-native type
  nullable: boolean;
  primary_key: boolean;
}

type DataValue =
  | { type: "Null" }
  | { type: "Boolean", value: boolean }
  | { type: "Integer", value: number }
  | { type: "Float", value: number }
  | { type: "String", value: string }
  | { type: "Bytes", value: number[] }
  | { type: "Date", value: string }      // ISO 8601
  | { type: "DateTime", value: string }  // ISO 8601
  | { type: "Json", value: any }
  | { type: "Array", value: DataValue[] };
```

#### Parameterized Queries

SQL databases support parameterized queries to prevent injection:

```typescript
// PostgreSQL uses $1, $2, etc.
const result = await executeQuery(
  connectionId,
  "SELECT * FROM users WHERE email = $1 AND active = $2",
  [
    { type: "String", value: "user@example.com" },
    { type: "Boolean", value: true }
  ]
);

// MySQL uses ? placeholders
const result = await executeQuery(
  connectionId,
  "SELECT * FROM users WHERE email = ? AND active = ?",
  [
    { type: "String", value: "user@example.com" },
    { type: "Boolean", value: true }
  ]
);
```

---

### 3. Schema Introspection

#### Get Schema

Retrieve complete schema metadata for a database:

```typescript
const schema = await getDatabaseSchema(connectionId, "production_db");

interface Schema {
  database_name: string;
  tables: Table[];
}

interface Table {
  name: string;
  schema?: string;        // Schema/database qualifier (PostgreSQL)
  columns: Column[];
  indexes: Index[];
  foreign_keys: ForeignKey[];
  row_count?: number;     // Approximate row count
}

interface Column {
  name: string;
  data_type: string;
  nullable: boolean;
  default_value?: string;
  primary_key: boolean;
  auto_increment: boolean;
}

interface Index {
  name: string;
  columns: string[];
  unique: boolean;
  index_type: string;     // "BTREE", "HASH", "GIN", etc.
}

interface ForeignKey {
  name: string;
  from_table: string;
  from_columns: string[];
  to_table: string;
  to_columns: string[];
  on_delete: string;      // "CASCADE", "SET NULL", "RESTRICT"
  on_update: string;
}
```

#### List Databases

Query available databases on the server:

```typescript
const databases = await listDatabases(connectionId);
// Returns: ["production_db", "staging_db", "analytics_db"]
```

---

### 4. Data Import/Export

#### CSV Import

Import CSV files into a table with automatic type inference:

```typescript
const stats = await importCsvData(
  "/path/to/data.csv",
  connectionId,
  "target_table",
  {
    skip_header: true,
    delimiter: ",",
    quote_char: "\"",
    batch_size: 1000,
    create_table: true,      // Auto-create table if missing
    truncate_first: false,   // Clear table before import
    type_inference: true     // Infer column types from data
  }
);

interface ImportStats {
  rows_imported: number;
  rows_skipped: number;
  errors: string[];
  execution_time_ms: number;
}
```

**CSV Format Requirements:**
- First row treated as headers (if `skip_header: true`)
- UTF-8 encoding
- Standard RFC 4180 format
- Supports quoted fields with embedded delimiters

**Type Inference:**
- Samples first 100 rows to infer types
- Falls back to `TEXT/VARCHAR` for ambiguous columns
- Respects explicit type hints in options

#### JSON Import

Import JSON arrays or objects into tables:

```typescript
// JSON array format
[
  { "id": 1, "name": "Alice", "email": "alice@example.com" },
  { "id": 2, "name": "Bob", "email": "bob@example.com" }
]

// JSON object with nested data array
{
  "metadata": { "export_date": "2025-06-29" },
  "data": [
    { "id": 1, "name": "Alice" }
  ]
}

const stats = await importJsonData(
  "/path/to/data.json",
  connectionId,
  "target_table",
  {
    create_table: true,
    truncate_first: false,
    batch_size: 500
  }
);
```

#### CSV Export

Export query results to CSV:

```typescript
const queryResult = await executeQuery(
  connectionId,
  "SELECT * FROM incidents WHERE severity = 'Critical'"
);

const stats = await exportQueryResults(
  queryResult,
  "csv",
  "/path/to/export.csv"
);

interface ExportStats {
  rows_exported: number;
  file_size_bytes: number;
  execution_time_ms: number;
}
```

#### JSON Export

Export query results to JSON array:

```typescript
await exportQueryResults(
  queryResult,
  "json",
  "/path/to/export.json"
);

// Output format:
[
  { "id": 1, "title": "Database outage", "severity": "Critical" },
  { "id": 2, "title": "API timeout", "severity": "Critical" }
]
```

#### SQL Export

Export query results as SQL INSERT statements:

```typescript
await exportQueryResults(
  queryResult,
  "sql",
  "/path/to/export.sql",
  "incidents"  // Table name for INSERT statements
);

// Output format:
-- Generated by TRCAA v3.0.0
-- Export Date: 2025-06-29T10:30:00Z

INSERT INTO incidents (id, title, severity) VALUES (1, 'Database outage', 'Critical');
INSERT INTO incidents (id, title, severity) VALUES (2, 'API timeout', 'Critical');
```

#### Preview Files

Preview CSV/JSON files before import:

```typescript
// Preview CSV (first 100 rows by default)
const preview = await previewCsvFile("/path/to/data.csv", 100);

interface PreviewData {
  headers: string[];
  rows: string[][];  // 2D array of string values
}

// Preview JSON (first 100 records)
const preview = await previewJsonFile("/path/to/data.json", 100);
// Returns: JSON value (array or object)
```

---

### 5. ER Diagram Generation

Automatically generate Entity-Relationship diagrams showing table structure and relationships.

```typescript
const erData = await generateErDiagram(connectionId, "production_db");

interface ERDiagramData {
  tables: ERTable[];
  relationships: ERRelationship[];
}

interface ERTable {
  name: string;
  columns: ERColumn[];
  position?: { x: number; y: number };  // Optional layout hints
}

interface ERColumn {
  name: string;
  type: string;
  is_primary: boolean;
  is_foreign: boolean;
  nullable: boolean;
}

interface ERRelationship {
  from_table: string;
  from_column: string;
  to_table: string;
  to_column: string;
  relationship_type: "one-to-one" | "one-to-many" | "many-to-many";
  constraint_name: string;
}
```

**ER Diagram Features:**

- **Auto-layout** — Force-directed graph algorithm for optimal spacing
- **Relationship Detection** — Extracts foreign key constraints and indexes
- **Cardinality Inference** — Detects one-to-one vs. one-to-many relationships
- **Visual Styling** — Primary keys highlighted, nullable columns indicated
- **Export to PNG/SVG** — Save diagrams for documentation

**Usage in Incident Response:**

1. **Schema Discovery** — Understand production database structure during outages
2. **Impact Analysis** — Visualize affected tables and dependencies
3. **Documentation** — Generate schema diagrams for post-mortem reports
4. **Query Planning** — Identify join paths and optimize queries

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────┐
│                   Frontend (React)                       │
│  DatabaseQueryPage.tsx → tauriCommands.ts               │
└───────────────────────┬─────────────────────────────────┘
                        │ IPC (invoke)
┌───────────────────────▼─────────────────────────────────┐
│              Rust Backend (Tauri)                        │
│  commands/database.rs → db_drivers/pool.rs              │
└───────────────────────┬─────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
┌───────▼───────┐ ┌────▼────┐ ┌────────▼────────┐
│ PostgreSQL    │ │  MySQL  │ │    MongoDB      │
│ Driver        │ │ Driver  │ │    Driver       │
└───────────────┘ └─────────┘ └─────────────────┘
        │               │               │
┌───────▼───────┐ ┌────▼────┐
│ Redis Driver  │ │Cassandra│
└───────────────┘ └─────────┘
```

### Module Structure

```
src-tauri/src/
├── commands/
│   └── database.rs           # IPC command handlers
├── db_drivers/
│   ├── mod.rs                # Driver factory
│   ├── traits.rs             # DatabaseDriver trait
│   ├── types.rs              # Common data types
│   ├── error.rs              # Error types
│   ├── pool.rs               # Connection pool manager
│   ├── postgres/
│   │   ├── driver.rs         # PostgreSQL implementation
│   │   └── types.rs          # PostgreSQL-specific types
│   ├── mysql/
│   │   ├── driver.rs         # MySQL implementation
│   │   └── types.rs          # MySQL-specific types
│   ├── mongodb/
│   │   ├── driver.rs         # MongoDB implementation
│   │   ├── schema.rs         # Collection introspection
│   │   └── types.rs          # BSON type conversions
│   ├── redis/
│   │   ├── driver.rs         # Redis implementation
│   │   └── types.rs          # Redis value types
│   ├── cassandra/
│   │   └── driver.rs         # Cassandra/Scylla driver
│   ├── import_export/
│   │   ├── csv.rs            # CSV import/export
│   │   ├── json.rs           # JSON import/export
│   │   └── sql.rs            # SQL export
│   └── visualization/
│       └── er_diagram.rs     # ER diagram generation
└── state.rs                  # AppState with db_pool_manager
```

### Driver Trait

All database drivers implement the `DatabaseDriver` trait:

```rust
#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    // Connection lifecycle
    async fn connect(&mut self, config: &ConnectionConfig) -> DriverResult<()>;
    async fn disconnect(&mut self) -> DriverResult<()>;
    async fn test_connection(&self) -> DriverResult<ConnectionStatus>;

    // Query execution
    async fn execute_query(
        &self,
        query: &str,
        params: Vec<DataValue>
    ) -> DriverResult<QueryResult>;

    // Schema introspection
    async fn get_databases(&self) -> DriverResult<Vec<String>>;
    async fn get_schema(&self, database: &str) -> DriverResult<Schema>;

    // Transactions (if supported)
    async fn begin_transaction(&mut self) -> DriverResult<TransactionHandle>;
    async fn commit_transaction(&mut self, handle: &TransactionHandle) -> DriverResult<()>;
    async fn rollback_transaction(&mut self, handle: &TransactionHandle) -> DriverResult<()>;

    // Metadata
    fn database_type(&self) -> DatabaseType;
    fn supports_transactions(&self) -> bool;
    fn is_connected(&self) -> bool;
}
```

### Database-Specific Implementation Notes

#### PostgreSQL Driver

- **Connection:** Uses `tokio-postgres` with native async support
- **Transactions:** Full ACID support with nested savepoints
- **Schema:** Queries `information_schema` for metadata
- **Types:** Maps PostgreSQL types to `DataValue` enum
- **SSL:** Supports client certificates and server verification

#### MySQL Driver

- **Connection:** Uses `mysql_async` for async queries
- **Transactions:** Full ACID support with isolation levels
- **Schema:** Queries `information_schema.tables` and `information_schema.columns`
- **Types:** Handles MySQL type quirks (TINYINT as bool, ENUM, SET)
- **SSL:** Supports TLS with optional certificate validation

#### MongoDB Driver

- **Connection:** Uses official `mongodb` async driver
- **Transactions:** Limited support (replica set required, sessions-based)
- **Schema:** Infers schema from collection samples (no strict schema)
- **Types:** BSON to `DataValue` conversion (ObjectId, Date, embedded documents)
- **Queries:** Supports find, aggregate, and command execution

#### Redis Driver

- **Connection:** Uses `redis` crate with async runtime
- **Transactions:** Limited (MULTI/EXEC atomic blocks, no rollback)
- **Schema:** No schema introspection (key-based access)
- **Types:** String, Hash, List, Set, Sorted Set mapped to `DataValue`
- **Queries:** Command strings parsed and executed

#### Cassandra Driver

- **Connection:** Uses `scylla` driver (Rust-native CQL driver)
- **Transactions:** No transaction support (eventual consistency)
- **Schema:** Queries `system_schema` keyspace for table metadata
- **Types:** CQL types mapped to `DataValue` (UUID, Timestamp, collections)
- **Queries:** CQL prepared statements with parameter binding

---

## Security

### Credential Encryption

All database credentials are encrypted before storage:

- **Algorithm:** AES-256-GCM (Authenticated Encryption)
- **Key Derivation:** From `TRCAA_ENCRYPTION_KEY` environment variable
- **Key Length:** 256-bit (32 bytes)
- **Nonce:** Random 96-bit nonce per encryption
- **Storage Format:** `base64(nonce || ciphertext || tag)`

**Encryption Flow:**

```rust
// On save
let encrypted = encrypt_password(&plaintext_password)?;
// encrypted: "nonce_12bytes||ciphertext||tag_16bytes" (base64)

// On load
let plaintext = decrypt_password(&encrypted)?;
```

### SSL/TLS Support

All drivers support encrypted connections:

```typescript
const sslConfig = {
  enabled: true,
  ca_cert_path: "/path/to/ca.crt",
  client_cert_path: "/path/to/client.crt",
  client_key_path: "/path/to/client.key",
  verify_server: true  // Verify server certificate
};
```

**SSL Configuration by Database:**

| Database | SSL Support | Client Certs | Server Verification |
|----------|------------|--------------|---------------------|
| PostgreSQL | ✅ Full | ✅ Yes | ✅ Yes |
| MySQL | ✅ Full | ✅ Yes | ✅ Yes |
| MongoDB | ✅ Full | ✅ Yes | ✅ Yes |
| Redis | ✅ TLS | ❌ No | ✅ Yes |
| Cassandra | ✅ Full | ✅ Yes | ✅ Yes |

### Audit Logging

All database operations are logged to the audit trail:

```sql
-- audit_log table entries
{
  "action": "database_query_executed",
  "entity_type": "database_connection",
  "entity_id": "conn_12345",
  "details": {
    "query": "SELECT * FROM incidents WHERE id = $1",
    "params_count": 1,
    "row_count": 1,
    "execution_time_ms": 45
  },
  "user_id": "system",
  "timestamp": "2025-06-29T10:30:00Z"
}
```

**Logged Events:**
- Connection created/updated/deleted
- Query executed (with execution time)
- Schema introspection performed
- Import/export operations
- Transaction begin/commit/rollback

### Connection Security Best Practices

1. **Use Read-Only Accounts** — Grant minimum required privileges
2. **Enable SSL/TLS** — Encrypt all network traffic
3. **Rotate Credentials** — Change passwords regularly
4. **Audit Access** — Review audit logs for suspicious activity
5. **Restrict Network Access** — Use firewall rules and VPNs
6. **Separate Environments** — Use different credentials for prod/staging

---

## Troubleshooting

### Common Issues

#### "Connection refused" or "Network unreachable"

**Cause:** Database server not reachable from TRCAA instance.

**Solutions:**
- Verify host and port are correct
- Check firewall rules allow traffic on database port
- Confirm database server is running (`systemctl status postgresql`)
- Test connectivity with `telnet <host> <port>`

#### "Authentication failed"

**Cause:** Invalid credentials or missing permissions.

**Solutions:**
- Verify username and password are correct
- Check user has `CONNECT` privilege on database
- For PostgreSQL, verify `pg_hba.conf` allows authentication method
- For MySQL, check user host restrictions (`'user'@'%'` vs `'user'@'localhost'`)

#### "SSL connection required"

**Cause:** Server requires SSL but connection configured without it.

**Solutions:**
- Enable SSL in connection config
- Provide CA certificate if using self-signed cert
- Set `verify_server: false` for testing (not recommended for production)

#### "Query timeout"

**Cause:** Long-running query exceeded timeout limit.

**Solutions:**
- Optimize query with indexes
- Reduce result set size with `LIMIT`
- Increase query timeout in connection options
- Check for table locks or deadlocks

#### "Schema introspection failed"

**Cause:** User lacks permissions to query metadata tables.

**Solutions:**
- Grant `SELECT` on `information_schema` (PostgreSQL/MySQL)
- Grant `DESCRIBE` on keyspaces (Cassandra)
- For MongoDB, ensure user can run `listCollections` command

#### "Import failed: Type mismatch"

**Cause:** CSV data doesn't match target table schema.

**Solutions:**
- Use `create_table: true` to auto-create table
- Enable `type_inference: true` for automatic type detection
- Preview CSV with `previewCsvFile` to verify data format
- Check for NULL values in NOT NULL columns

### Error Codes

| Code | Error | Description | Resolution |
|------|-------|-------------|------------|
| `DB_001` | ConnectionFailed | Failed to establish connection | Check host, port, credentials |
| `DB_002` | AuthenticationError | Invalid credentials | Verify username/password |
| `DB_003` | QueryExecutionError | Query failed | Check query syntax and permissions |
| `DB_004` | SchemaNotFound | Database or table not found | Verify database name |
| `DB_005` | TransactionError | Transaction commit/rollback failed | Check database logs |
| `DB_006` | TimeoutError | Operation exceeded timeout | Optimize query or increase timeout |
| `DB_007` | PermissionDenied | User lacks required privileges | Grant necessary permissions |
| `DB_008` | SslError | SSL/TLS connection error | Verify certificates and SSL config |

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
# Set environment variable before starting TRCAA
export RUST_LOG=trcaa=debug,sqlx=debug,mongodb=debug
cargo tauri dev
```

**Log locations:**
- **Linux:** `~/.local/share/tftsr/logs/`
- **macOS:** `~/Library/Application Support/tftsr/logs/`
- **Windows:** `%APPDATA%\tftsr\logs\`

---

## API Reference

### IPC Commands

All database management commands are exposed via Tauri IPC. TypeScript wrappers in `src/lib/tauriCommands.ts`.

#### `import_csv_data`

Import CSV file into a database table.

```typescript
async function importCsvData(
  filePath: string,
  connectionId: string,
  targetTable: string,
  options?: ImportOptions
): Promise<ImportStats>
```

**Parameters:**
- `filePath` — Absolute path to CSV file
- `connectionId` — Database connection ID
- `targetTable` — Target table name
- `options` — Optional import configuration

**Options:**
```typescript
interface ImportOptions {
  skip_header?: boolean;      // Skip first row (default: true)
  delimiter?: string;          // Column delimiter (default: ",")
  quote_char?: string;         // Quote character (default: "\"")
  batch_size?: number;         // Insert batch size (default: 1000)
  create_table?: boolean;      // Auto-create table (default: false)
  truncate_first?: boolean;    // Clear table before import (default: false)
  type_inference?: boolean;    // Infer column types (default: true)
}
```

**Returns:**
```typescript
interface ImportStats {
  rows_imported: number;
  rows_skipped: number;
  errors: string[];
  execution_time_ms: number;
}
```

**Example:**
```typescript
const stats = await importCsvData(
  "/tmp/incidents.csv",
  "prod_db_conn",
  "incidents_import",
  {
    skip_header: true,
    create_table: true,
    batch_size: 500
  }
);
console.log(`Imported ${stats.rows_imported} rows in ${stats.execution_time_ms}ms`);
```

---

#### `import_json_data`

Import JSON file into a database table.

```typescript
async function importJsonData(
  filePath: string,
  connectionId: string,
  targetTable: string,
  options?: ImportOptions
): Promise<ImportStats>
```

**Parameters:** Same as `import_csv_data`

**Supported JSON formats:**
```javascript
// Array format
[
  { "id": 1, "name": "Alice" },
  { "id": 2, "name": "Bob" }
]

// Object with data array
{
  "metadata": { "export_date": "2025-06-29" },
  "data": [
    { "id": 1, "name": "Alice" }
  ]
}
```

---

#### `export_query_results`

Export query results to file.

```typescript
async function exportQueryResults(
  queryResult: QueryResult,
  format: "csv" | "json" | "sql",
  outputPath: string,
  tableName?: string
): Promise<ExportStats>
```

**Parameters:**
- `queryResult` — Query result from `executeQuery`
- `format` — Export format: "csv", "json", or "sql"
- `outputPath` — Absolute path for output file
- `tableName` — Required for SQL format (table name for INSERT statements)

**Returns:**
```typescript
interface ExportStats {
  rows_exported: number;
  file_size_bytes: number;
  execution_time_ms: number;
}
```

---

#### `generate_er_diagram`

Generate Entity-Relationship diagram for a database.

```typescript
async function generateErDiagram(
  connectionId: string,
  database?: string
): Promise<ERDiagramData>
```

**Parameters:**
- `connectionId` — Database connection ID
- `database` — Database name (optional, uses connection default)

**Returns:**
```typescript
interface ERDiagramData {
  tables: ERTable[];
  relationships: ERRelationship[];
}

interface ERTable {
  name: string;
  columns: ERColumn[];
  position?: { x: number; y: number };
}

interface ERColumn {
  name: string;
  type: string;
  is_primary: boolean;
  is_foreign: boolean;
  nullable: boolean;
}

interface ERRelationship {
  from_table: string;
  from_column: string;
  to_table: string;
  to_column: string;
  relationship_type: "one-to-one" | "one-to-many" | "many-to-many";
  constraint_name: string;
}
```

---

#### `preview_csv_file`

Preview first N rows of a CSV file.

```typescript
async function previewCsvFile(
  filePath: string,
  maxRows?: number
): Promise<PreviewData>
```

**Parameters:**
- `filePath` — Absolute path to CSV file
- `maxRows` — Maximum rows to preview (default: 100)

**Returns:**
```typescript
interface PreviewData {
  headers: string[];
  rows: string[][];  // 2D array of string values
}
```

---

#### `preview_json_file`

Preview first N records of a JSON file.

```typescript
async function previewJsonFile(
  filePath: string,
  maxRecords?: number
): Promise<any>
```

**Parameters:**
- `filePath` — Absolute path to JSON file
- `maxRecords` — Maximum records to preview (default: 100)

**Returns:** JSON value (array or object)

---

## Developer Guide

### Adding a New Database Driver

To add support for a new database system:

#### 1. Create Driver Module

```rust
// src-tauri/src/db_drivers/newdb/mod.rs
pub mod driver;
pub mod types;

pub use driver::NewDbDriver;
```

#### 2. Implement DatabaseDriver Trait

```rust
// src-tauri/src/db_drivers/newdb/driver.rs
use crate::db_drivers::{
    error::DriverResult,
    traits::{ConnectionStatus, DatabaseDriver},
    types::*,
};
use async_trait::async_trait;

pub struct NewDbDriver {
    config: ConnectionConfig,
    client: Option<NewDbClient>,
}

impl NewDbDriver {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }
}

#[async_trait]
impl DatabaseDriver for NewDbDriver {
    async fn connect(&mut self, config: &ConnectionConfig) -> DriverResult<()> {
        // Create client connection
        let client = NewDbClient::connect(&config.host, config.port).await?;
        self.client = Some(client);
        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        if let Some(client) = self.client.take() {
            client.close().await?;
        }
        Ok(())
    }

    async fn test_connection(&self) -> DriverResult<ConnectionStatus> {
        // Ping database to verify connectivity
        let client = self.client.as_ref()
            .ok_or_else(|| DriverError::NotConnected)?;
        
        let start = std::time::Instant::now();
        client.ping().await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(ConnectionStatus {
            is_connected: true,
            message: "Connected".to_string(),
            server_version: Some(client.version().await?),
            latency_ms: Some(latency_ms),
        })
    }

    async fn execute_query(
        &self,
        query: &str,
        params: Vec<DataValue>
    ) -> DriverResult<QueryResult> {
        // Execute query and map results to QueryResult
        let client = self.client.as_ref()
            .ok_or_else(|| DriverError::NotConnected)?;
        
        let start = std::time::Instant::now();
        let result = client.query(query, params).await?;
        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(QueryResult {
            columns: result.columns,
            rows: result.rows,
            row_count: result.rows.len(),
            execution_time_ms,
        })
    }

    // Implement remaining trait methods...

    fn database_type(&self) -> DatabaseType {
        self.config.database_type
    }

    fn supports_transactions(&self) -> bool {
        true  // or false if not supported
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}
```

#### 3. Add to Factory

```rust
// src-tauri/src/db_drivers/mod.rs
pub mod newdb;

pub fn create_driver(config: &ConnectionConfig) -> DriverResult<Box<dyn DatabaseDriver>> {
    match config.database_type {
        DatabaseType::NewDb => {
            let driver = newdb::NewDbDriver::new(config.clone());
            Ok(Box::new(driver))
        }
        // ... existing drivers
    }
}
```

#### 4. Add Database Type

```rust
// src-tauri/src/db_drivers/types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MongoDB,
    Redis,
    Cassandra,
    NewDb,  // Add new type
}

impl DatabaseType {
    pub fn default_port(&self) -> u16 {
        match self {
            Self::NewDb => 7777,  // Add default port
            // ... existing types
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "newdb" => Some(Self::NewDb),  // Add string mapping
            // ... existing mappings
        }
    }
}
```

#### 5. Write Tests

```rust
// src-tauri/src/db_drivers/newdb/driver.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_newdb_connect() {
        let config = ConnectionConfig {
            database_type: DatabaseType::NewDb,
            host: "localhost".to_string(),
            port: 7777,
            database: Some("test".to_string()),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            ssl_config: None,
            options: HashMap::new(),
        };

        let mut driver = NewDbDriver::new(config.clone());
        driver.connect(&config).await.unwrap();
        assert!(driver.is_connected());
    }
}
```

### Type Mapping Guidelines

Map database-native types to `DataValue` enum:

| Database Type | DataValue Type | Notes |
|---------------|---------------|-------|
| INTEGER, BIGINT | `Integer(i64)` | All integer types → i64 |
| REAL, DOUBLE | `Float(f64)` | All floating-point → f64 |
| VARCHAR, TEXT | `String(String)` | All text types → String |
| BOOLEAN | `Boolean(bool)` | Boolean values |
| DATE | `Date(String)` | ISO 8601 date string |
| TIMESTAMP | `DateTime(String)` | ISO 8601 datetime string |
| BYTEA, BLOB | `Bytes(Vec<u8>)` | Binary data |
| JSON, JSONB | `Json(serde_json::Value)` | JSON values |
| ARRAY | `Array(Vec<DataValue>)` | Arrays of values |
| NULL | `Null` | Null values |

### Testing Checklist

Before submitting a new driver:

- [ ] All `DatabaseDriver` trait methods implemented
- [ ] Connection lifecycle tested (connect/disconnect)
- [ ] Query execution tested with sample queries
- [ ] Schema introspection returns correct metadata
- [ ] Type conversions handle all database types
- [ ] Error handling covers common failure cases
- [ ] SSL/TLS support tested (if applicable)
- [ ] Transaction support tested (if applicable)
- [ ] Integration tests pass against real database
- [ ] Documentation updated with new database type

---

## MySQL GPL Compliance

TRCAA uses the `mysql_async` crate which links to MySQL client libraries. See [MYSQL_LICENSE.md](/docs/MYSQL_LICENSE.md) for GPL compliance information.

---

## See Also

- [IPC Commands](/docs/wiki/IPC-Commands.md) — Complete IPC command reference
- [Architecture](/docs/wiki/Architecture.md) — System architecture overview
- [Security Model](/docs/wiki/Security-Model.md) — Security and encryption details
- [Troubleshooting](/docs/wiki/Troubleshooting.md) — Common issues and solutions
