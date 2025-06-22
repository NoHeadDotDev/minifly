# CLAUDE.md - AI Assistant Context for Minifly

## Project Overview
Minifly is a local Fly.io development simulator that provides incredible developer experience for testing Fly.io applications locally.

## Testing Infrastructure

### Test Commands
```bash
# Run all tests
cargo test --workspace

# Run with nextest (preferred)
cargo nextest run --workspace

# Run specific module tests
cargo test -p minifly-api config::tests

# Generate coverage report
cargo llvm-cov nextest --workspace --lcov --output-path lcov.info

# Generate coverage for specific modules
cargo llvm-cov test -p minifly-core -p minifly-api --lcov --output-path coverage.lcov
```

### Test Structure
- **Unit Tests**: Located in `src/` files within `#[cfg(test)]` modules
- **Integration Tests**: Located in `tests/` directories for each crate
- **CLI Tests**: In `minifly-cli/tests/` using `assert_cmd` and `predicates`

### Current Test Status
- ✅ Unit tests for `minifly-api` config module
- ✅ Unit tests for `minifly-core` models  
- ⚠️  Integration tests temporarily disabled (need Docker/DB mocking)
- ⚠️  Some CLI tests need updates for interface changes

### Testing Libraries
- `tokio-test` - Async test utilities
- `wiremock` - HTTP mocking
- `serial_test` - Sequential test execution
- `assert_cmd` - CLI testing
- `predicates` - Assertion predicates
- `cargo-llvm-cov` - Coverage reporting
- `cargo-nextest` - Fast test runner

### Known Issues
1. Integration tests require mock implementations for:
   - Docker client (`bollard`)
   - SQLite database (`sqlx`)
   - LiteFS manager

2. CLI tests need updates for:
   - `--version` flag support
   - Subcommand naming (e.g., "machine" vs "machines")

### Next Steps for Testing
1. Implement mock traits for external dependencies
2. Create integration tests with test containers
3. Update CLI tests to match current interface
4. Add property-based tests for complex logic
5. Set up CI/CD with coverage reporting

## Development Tips
- Use `serial` attribute for tests that modify environment variables
- Run `cargo nextest` for faster test execution
- Check coverage with `cargo llvm-cov` to identify untested code
- Integration tests should use ephemeral ports (port 0)

## Important Files
- `/tests/README.md` - Detailed testing documentation
- `minifly-api/src/config.rs` - Example of well-tested module
- `minifly-core/src/lib.rs` - Example unit tests for models