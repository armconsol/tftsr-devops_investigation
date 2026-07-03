# Database Management Integration Tests

This directory contains integration tests for the v3.0.0 database management feature.

## Overview

The test suite validates all 5 database drivers (PostgreSQL, MySQL, MongoDB, Redis, Cassandra) with real database instances running in Docker containers.

## Prerequisites

- Docker and Docker Compose installed
- Rust toolchain (for running integration tests)
- At least 4GB RAM available for Docker

## Quick Start

### 1. Start Test Databases

```bash
cd tests
docker-compose up -d
```

Wait for all health checks to pass (30-60 seconds):

```bash
docker-compose ps
```

All services should show status "healthy".

### 2. Run Integration Tests

```bash
# Run all integration tests
cargo test --test integration -- --test-threads=1

# Run specific database tests
cargo test --test integration postgres_driver -- --test-threads=1
cargo test --test integration mongodb_driver -- --test-threads=1
cargo test --test integration redis_driver -- --test-threads=1
```

### 3. Stop Test Databases

```bash
cd tests
docker-compose down -v  # -v removes volumes
```

## Test Structure

```
tests/
├── docker-compose.yml          # All 5 test database containers
├── fixtures/                   # Test data initialization scripts
│   ├── postgres_test_data.sql
│   ├── mysql_test_data.sql
│   ├── mongodb_test_data.js
│   ├── redis_test_data.sh
│   └── cassandra_test_data.cql
└── README.md                   # This file

src-tauri/tests/
├── integration/                # Integration test files
│   ├── postgres_driver_test.rs
│   ├── mysql_driver_test.rs
│   ├── mongodb_driver_test.rs
│   ├── redis_driver_test.rs
│   ├── cassandra_driver_test.rs
│   └── import_export_test.rs
└── common/                     # Shared test utilities
    └── mod.rs
```

## Test Coverage

### PostgreSQL (15 tests)
- Connection management (connect, disconnect, test)
- Query execution (SELECT, INSERT, UPDATE, DELETE)
- Type conversion (all PostgreSQL types)
- Schema introspection (tables, columns, indexes, foreign keys)
- Transaction management (BEGIN, COMMIT, ROLLBACK)

### MySQL (15 tests)
- Connection management
- Query execution with all MySQL types
- Schema introspection
- Transaction support
- Error handling

### MongoDB (12 tests)
- Connection with authentication
- Document queries (find, insertMany, updateMany)
- Aggregation pipelines
- Schema inference from documents
- BSON type conversion

### Redis (10 tests)
- Connection with password
- String operations (GET, SET, INCR, DECR)
- List operations (LPUSH, LPOP, LRANGE)
- Hash operations (HSET, HGET, HGETALL)
- Set operations (SADD, SMEMBERS)
- Key management (KEYS, DEL, EXISTS)

### Cassandra (12 tests)
- Connection to keyspace
- CQL query execution
- Type conversion (text, int, uuid, timestamp)
- Schema introspection
- Prepared statements

### Import/Export (8 tests)
- CSV import with various data types
- JSON import with nested structures
- SQL export generation
- Error handling (malformed files, type mismatches)
- Progress reporting

## Database Connection Details

### PostgreSQL
- Host: localhost
- Port: 5432
- Database: testdb
- User: testuser
- Password: testpassword

### MySQL
- Host: localhost
- Port: 3306
- Database: testdb
- User: testuser
- Password: testpassword

### MongoDB
- Host: localhost
- Port: 27017
- Database: testdb
- User: testuser
- Password: testpassword
- Auth Source: admin

### Redis
- Host: localhost
- Port: 6379
- Password: testpassword
- Database: 0

### Cassandra
- Host: localhost
- Port: 9042
- Keyspace: test_keyspace
- No authentication (test environment)

## Troubleshooting

### "Connection refused" errors

**Problem**: Tests fail with connection refused

**Solution**: Ensure Docker containers are running and healthy

```bash
docker-compose ps  # Check status
docker-compose logs postgres  # Check logs for specific service
```

### "Port already in use" errors

**Problem**: Cannot start containers because ports are occupied

**Solution**: Stop conflicting services or change ports in docker-compose.yml

```bash
# Check what's using port 5432
lsof -i :5432

# Stop local PostgreSQL
sudo systemctl stop postgresql
```

### Slow container startup

**Problem**: Health checks take too long

**Solution**: Cassandra especially can take 60+ seconds to start. Wait for all health checks:

```bash
watch docker-compose ps
```

### Tests fail randomly

**Problem**: Race conditions or timing issues

**Solution**: Run tests with `--test-threads=1` to avoid parallel execution:

```bash
cargo test --test integration -- --test-threads=1
```

### Permission denied errors

**Problem**: Cannot write to docker volumes

**Solution**: Check Docker permissions or run with appropriate user:

```bash
# Linux: Add user to docker group
sudo usermod -aG docker $USER
newgrp docker
```

### MongoDB authentication fails

**Problem**: Cannot connect with testuser/testpassword

**Solution**: Wait for MongoDB to fully initialize (check logs):

```bash
docker-compose logs mongodb
```

If authentication continues to fail, recreate the container:

```bash
docker-compose down -v
docker-compose up -d mongodb
```

## CI/CD Integration

### GitHub Actions / Gitea Actions

The test suite integrates with CI/CD via `test-database.yml` workflow:

```yaml
name: Database Integration Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        ...
      mysql:
        image: mysql:8.0
        ...
      # ... other databases
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Run tests
        run: cargo test --test integration -- --test-threads=1
```

## Performance Benchmarks

Expected test execution times on modern hardware:

| Test Suite | Duration | Notes |
|------------|----------|-------|
| PostgreSQL | ~15s | 15 tests with real queries |
| MySQL | ~12s | 15 tests |
| MongoDB | ~10s | 12 tests |
| Redis | ~5s | 10 tests, very fast |
| Cassandra | ~20s | 12 tests, slower startup |
| Import/Export | ~8s | 8 tests with file I/O |
| **Total** | **~70s** | All 72 tests |

## Writing New Tests

### Test Template

```rust
#[tokio::test]
async fn test_postgres_basic_query() {
    // Setup
    let config = common::setup_postgres_config();
    let mut driver = PostgresDriver::new(config.clone());
    driver.connect(&config).await.expect("Failed to connect");

    // Execute
    let result = driver
        .execute_query("SELECT * FROM users WHERE active = true", vec![])
        .await
        .expect("Query failed");

    // Assert
    assert!(result.row_count > 0, "Should return active users");
    assert_eq!(result.columns.len(), 6, "Users table has 6 columns");

    // Cleanup
    driver.disconnect().await.expect("Failed to disconnect");
}
```

### Helper Functions

See `src-tauri/tests/common/mod.rs` for shared utilities:

- `setup_postgres_config()` - PostgreSQL connection config
- `setup_mysql_config()` - MySQL connection config
- `setup_mongodb_config()` - MongoDB connection config
- `wait_for_postgres()` - Wait for PostgreSQL readiness
- `cleanup_test_data()` - Reset test data between runs

## Additional Resources

- [Main Documentation](../docs/wiki/Database-Management.md)
- [Architecture Guide](../docs/wiki/Architecture.md)
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Docker Compose Documentation](https://docs.docker.com/compose/)

## Support

For issues with the test suite:
1. Check this troubleshooting guide
2. Review test output for specific error messages
3. Examine Docker logs: `docker-compose logs <service>`
4. Report persistent issues via the project issue tracker
