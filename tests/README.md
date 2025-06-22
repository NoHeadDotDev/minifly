# Minifly Testing Infrastructure

This document describes the testing setup for the Minifly project.

## Test Structure

### Unit Tests
- Located in `src/` files using `#[cfg(test)]` modules
- Example: `minifly-api/src/config.rs` contains unit tests for configuration parsing
- Use `serial_test` for tests that modify environment variables to avoid race conditions

### Integration Tests
- Located in `tests/` directories for each crate
- Currently disabled pending mock infrastructure implementation
- Will test full API endpoints and CLI commands

### Test Dependencies
- `tokio-test` - Testing async code
- `wiremock` - HTTP mocking
- `tempfile` - Temporary file/directory creation
- `serial_test` - Serial test execution
- `pretty_assertions` - Better assertion output
- `assert_cmd` - CLI testing
- `predicates` - Command output assertions

## Running Tests

### Run all tests
```bash
cargo test --workspace
```

### Run with cargo nextest (recommended)
```bash
cargo nextest run --workspace
```

### Run specific test module
```bash
cargo test -p minifly-api config::tests
```

## Coverage

### Generate coverage report
```bash
cargo llvm-cov nextest --workspace --lcov --output-path lcov.info
```

### Generate coverage for specific module
```bash
cargo llvm-cov test -p minifly-api config::tests --lcov --output-path minifly-api-config.lcov
```

## Known Issues

1. **Integration tests temporarily disabled** - The minifly-api integration tests require mock implementations for Docker and database connections. These are commented out until proper test infrastructure is implemented.

2. **CLI tests need updates** - Some CLI tests fail due to interface changes. These need to be updated to match the current CLI structure.

## Future Improvements

1. **Mock Infrastructure** - Implement proper mocking for:
   - Docker client
   - SQLite database 
   - LiteFS manager

2. **E2E Tests** - Add end-to-end tests that:
   - Start the API server
   - Create apps and machines
   - Verify Docker containers are created
   - Test full workflows

3. **Performance Tests** - Add benchmarks for critical paths

4. **Property-based Tests** - Consider adding proptest for complex logic

## Example Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    
    #[test]
    #[serial]
    fn test_config_parsing() {
        // Clear environment to ensure clean state
        std::env::remove_var("MINIFLY_API_PORT");
        
        // Test default values
        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 4280);
        
        // Test custom values
        std::env::set_var("MINIFLY_API_PORT", "8080");
        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 8080);
        
        // Clean up
        std::env::remove_var("MINIFLY_API_PORT");
    }
}
```