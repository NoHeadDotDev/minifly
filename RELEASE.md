# Release Management for Minifly

## Current Status
- Development Version: 0.1.1
- Last Published: 0.1.1 (to crates.io)
- GitHub: Active development

## Release Strategy

### 1. Version Management
The project uses workspace-level version management in the root `Cargo.toml`:
```toml
[workspace.package]
version = "0.1.1"
```

### 2. Development vs Release Branches
For active development without publishing to crates.io:

#### Option A: Feature Branch Strategy (Recommended)
```bash
# Create a development branch
git checkout -b dev

# Work on features
git add .
git commit -m "Add new feature"
git push origin dev

# When ready for release
git checkout main
git merge dev
cargo publish --dry-run  # Test publish
cargo publish
```

#### Option B: Pre-release Versions
Update version to indicate pre-release:
```toml
[workspace.package]
version = "0.2.0-dev"  # or "0.2.0-alpha.1"
```

### 3. Preventing Accidental Publishing

#### Add to Cargo.toml (if needed):
```toml
[package]
publish = false  # Prevents publishing this specific crate
```

#### Or use publish field selectively:
```toml
[package]
publish = ["crates-io"]  # Default: can publish
# publish = []  # Empty array: cannot publish
```

### 4. Release Checklist

Before publishing to crates.io:
- [ ] All tests passing (`cargo test --workspace`)
- [ ] Update version in root `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run `cargo publish --dry-run` for each crate
- [ ] Tag the release: `git tag v0.2.0`
- [ ] Push tag: `git push origin v0.2.0`
- [ ] Publish in dependency order:
  1. `cd minifly-core && cargo publish`
  2. `cd ../minifly-logging && cargo publish`
  3. `cd ../minifly-network && cargo publish`
  4. `cd ../minifly-litefs && cargo publish`
  5. `cd ../minifly-api && cargo publish`
  6. `cd ../minifly-cli && cargo publish`
  7. `cd ../minifly && cargo publish`

### 5. Current Development Workflow

Since we're not ready to publish yet:

1. **Continue development on main branch**
   - Regular commits and pushes to GitHub
   - Version remains at 0.1.1 in Cargo.toml

2. **When ready for next release**:
   - Bump version to 0.2.0 (or appropriate)
   - Follow release checklist above
   - Publish all crates in order

3. **For experimental features**:
   - Use feature branches
   - Test thoroughly before merging

### 6. Using Git Tags for Versions

Mark development milestones without publishing:
```bash
# Tag development versions
git tag v0.2.0-dev.1
git push origin v0.2.0-dev.1

# Tag release candidates
git tag v0.2.0-rc.1
git push origin v0.2.0-rc.1

# Tag final releases
git tag v0.2.0
git push origin v0.2.0
```

## Summary

For now, continue development on the main branch. The crates.io version (0.1.1) remains stable and available for users, while active development continues on GitHub. When ready for the next release, bump the version and follow the release checklist.