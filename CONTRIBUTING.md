# Contributing to PolicyCheck

Thank you for your interest in contributing to PolicyCheck! This document provides guidelines and information for contributors.

## Code of Conduct

PolicyCheck is part of the [OpenAttribution](https://openattribution.org) initiative. We are committed to providing a welcoming and inclusive environment for all contributors.

## Getting Started

### Prerequisites

- Rust 1.75+ ([Install Rust](https://rustup.rs/))
- Git
- Basic familiarity with Rust and command line

### Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/policycheck.git
   cd policycheck
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/openattribution-org/policycheck.git
   ```
4. Install dependencies:
   ```bash
   cargo build
   ```

## Development Workflow

### Before Making Changes

1. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Keep your branch up to date:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

### Making Changes

1. **Write tests first** (TDD approach recommended)
2. Implement your changes
3. Ensure all tests pass:
   ```bash
   cargo test
   ```
4. Format your code:
   ```bash
   cargo fmt
   ```
5. Run clippy for linting:
   ```bash
   cargo clippy -- -D warnings
   ```

**Pro tip:** Set up a pre-commit hook to run checks automatically:
```bash
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
EOF
chmod +x .git/hooks/pre-commit
```

### Testing Requirements

All contributions must include tests:

- **Unit tests**: For individual functions/methods
- **Integration tests**: For end-to-end workflows (when applicable)
- **Test coverage**: Aim for >80% coverage for new code

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = "test input";

        // Act
        let result = your_function(input);

        // Assert
        assert_eq!(result, expected_output);
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in release mode
cargo test --release
```

### Code Style

We follow Rust standard style guidelines:

- Use `cargo fmt` to format code
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Write clear, descriptive variable and function names
- Add comments for complex logic
- Document public APIs with doc comments

Example:
```rust
/// Extracts RSL licenses from robots.txt content
///
/// # Arguments
///
/// * `content` - The robots.txt file content as a string
///
/// # Returns
///
/// A tuple of (global_licenses, group_licenses)
fn extract_licenses(&self, content: &str) -> (Vec<String>, Vec<String>) {
    // Implementation
}
```

## Commit Guidelines

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(rsl): add support for multiple license URIs

Implements extraction of multiple RSL license directives
from a single user-agent group.

Closes #123
```

```
fix(analyzer): correct license precedence logic

Group-scoped licenses should override global licenses
per RSL specification.
```

### Signing Commits

Please sign your commits with GPG:
```bash
git commit -S -m "your message"
```

## Pull Request Process

1. **Update documentation** if you've changed APIs or added features
2. **Ensure CI passes** - all tests, linting, and formatting checks
3. **Request review** from maintainers
4. **Address feedback** promptly and professionally

### PR Checklist

Before opening a PR, ensure:

- [ ] Tests added/updated and passing (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Documentation updated (if applicable)
- [ ] Commit messages follow conventional commits
- [ ] PR description clearly explains the change

**The CI will automatically run all these checks** when you open a PR.

### PR Template

When opening a PR, include:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How has this been tested?

## Checklist
- [ ] Tests pass (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Documentation updated (if applicable)
```

## Areas for Contribution

### High Priority

1. **TDM Policy Support** - Implement `.well-known/tdmrep.json` checking
2. **security.txt Support** - Add security contact discovery
3. **Integration Tests** - Expand test coverage
4. **Documentation** - Improve examples and guides

### Good First Issues

Look for issues tagged with `good first issue` in the issue tracker.

### Ideas for Enhancement

- Caching layer for repeated checks
- Web UI dashboard
- Additional output formats (YAML, XML)
- Plugin system for custom analyzers
- Performance optimizations

## Standards Compliance

When implementing new features:

- **Follow specifications** closely (RFC 9309, RSL, etc.)
- **Test edge cases** thoroughly
- **Document deviations** from specs (if any)
- **Add references** to relevant spec sections

## Questions?

- Open a [Discussion](https://github.com/openattribution-org/policycheck/discussions)
- Join the OpenAttribution community
- Check existing [Issues](https://github.com/openattribution-org/policycheck/issues)

## License

By contributing to PolicyCheck, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to PolicyCheck and the OpenAttribution initiative!** 🔍
