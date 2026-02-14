# CI/CD Setup for PolicyCheck

## ✅ Complete Test Suite

### Unit Tests (8 tests)

All tests passing in `src/analyzer.rs`:

1. ✅ `test_extract_user_agents` - User agent parsing
2. ✅ `test_extract_paths` - Allow/Disallow path extraction
3. ✅ `test_extract_global_licenses` - Global RSL license detection
4. ✅ `test_extract_group_scoped_licenses` - Group-scoped RSL licenses
5. ✅ `test_license_precedence_group_overrides_global` - RSL precedence rules
6. ✅ `test_license_requires_absolute_uri` - URI validation
7. ✅ `test_license_ignores_comments` - Comment handling
8. ✅ `test_wildcard_user_agent_matches` - Wildcard matching

### Running Tests Locally

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_extract_licenses

# Run in release mode
cargo test --release
```

## 🚀 GitHub Actions Workflows

### 1. CI Workflow (`.github/workflows/ci.yml`)

**Triggers**: Push to main/master, Pull Requests

**Jobs**:

- **Test** - Runs on Ubuntu, macOS, Windows with Rust stable and 1.85.0
  - Executes test suite
  - Caches cargo dependencies for speed

- **Clippy** - Linting
  - Runs clippy with all warnings as errors
  - Ensures code quality

- **Format** - Code formatting
  - Checks rustfmt compliance
  - Enforces consistent style

- **Build** - Release builds
  - Builds on all platforms
  - Uploads artifacts (binaries)

- **Security** - Security audit
  - Runs `cargo audit`
  - Checks for vulnerable dependencies

**Badge**:
```markdown
![CI](https://github.com/openattribution-org/policycheck/workflows/CI/badge.svg)
```

### 2. Release Workflow (`.github/workflows/release.yml`)

**Triggers**: Git tags matching `v*` (e.g., `v0.1.0`)

**Jobs**:

- **Create Release** - Creates GitHub release

- **Build Release** - Multi-platform binaries
  - Linux x86_64
  - Linux ARM64 (aarch64)
  - macOS x86_64 (Intel)
  - macOS ARM64 (Apple Silicon)
  - Windows x86_64

- **Publish Crate** - Publishes to crates.io
  - Requires `CARGO_REGISTRY_TOKEN` secret

**Creating a Release**:
```bash
# Tag version
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# Workflow automatically:
# 1. Creates GitHub release
# 2. Builds binaries for all platforms
# 3. Uploads release assets
# 4. Publishes to crates.io
```

### 3. Documentation Workflow (`.github/workflows/docs.yml`)

**Triggers**: Push to main/master

**Jobs**:

- **Generate Docs** - Builds rustdoc documentation
- **Deploy to GitHub Pages** - Publishes docs

**Viewing Docs**:
```
https://openattribution-org.github.io/policycheck/
```

## 📋 Required GitHub Secrets

For full CI/CD functionality, configure these secrets in GitHub Settings:

1. **`CARGO_REGISTRY_TOKEN`** - For publishing to crates.io
   - Get from: https://crates.io/settings/tokens
   - Scope: `publish-update`

## 🔧 Pre-commit Checks

Before pushing, run these locally:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Build release
cargo build --release
```

Or use this one-liner:
```bash
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test && cargo build --release
```

## 📝 Contributing Guidelines

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development workflow
- Testing requirements
- Code style guidelines
- Commit message format
- PR process

## 🏷️ Version Tagging Strategy

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (v1.0.0): Breaking changes
- **MINOR** (v0.1.0): New features, backwards compatible
- **PATCH** (v0.0.1): Bug fixes, backwards compatible

Example release process:
```bash
# Update version in Cargo.toml
# Update CHANGELOG.md

git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin main --tags
```

## 📊 CI Status Badges

Add to README.md:

```markdown
[![CI](https://github.com/openattribution-org/policycheck/workflows/CI/badge.svg)](https://github.com/openattribution-org/policycheck/actions/workflows/ci.yml)
[![Release](https://github.com/openattribution-org/policycheck/workflows/Release/badge.svg)](https://github.com/openattribution-org/policycheck/actions/workflows/release.yml)
[![Documentation](https://github.com/openattribution-org/policycheck/workflows/Documentation/badge.svg)](https://openattribution-org.github.io/policycheck/)
```

## 🔍 Monitoring CI

### View CI Runs
- Go to: https://github.com/openattribution-org/policycheck/actions
- See all workflow runs, logs, and artifacts

### Common CI Failures

**Tests Failing**:
- Run `cargo test` locally
- Check for platform-specific issues
- Review test output in CI logs

**Clippy Warnings**:
- Run `cargo clippy --all-targets --all-features`
- Fix all warnings before pushing
- Some warnings can be allowed with `#[allow(clippy::...)]`

**Format Issues**:
- Run `cargo fmt --all`
- Commit formatting changes

**Build Failures**:
- Check Cargo.toml dependencies
- Verify Rust version compatibility
- Review build logs

## 🎯 Quality Gates

PRs must pass all checks:
- ✅ All tests passing
- ✅ No clippy warnings
- ✅ Code formatted
- ✅ Builds successfully
- ✅ No security vulnerabilities

## 📈 Future Enhancements

Potential CI/CD improvements:

- [ ] Code coverage reporting (codecov.io)
- [ ] Benchmark performance tracking
- [ ] Docker image builds
- [ ] Nightly Rust testing
- [ ] Fuzz testing
- [ ] Integration test suite
- [ ] Dependabot for dependency updates

## 🚀 Quick Start for New Contributors

1. Fork the repo
2. Clone your fork
3. Create feature branch
4. Make changes + add tests
5. Run pre-commit checks (format, clippy, test)
6. Push and open PR
7. CI runs automatically
8. Address any CI failures
9. Get review and merge

## 📚 Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://matklad.github.io/2021/09/04/fast-rust-builds.html)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)

---

**CI/CD is fully configured and ready for production! 🎉**
