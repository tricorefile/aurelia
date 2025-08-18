# Contributing to Aurelia

Thank you for your interest in contributing to Aurelia! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

Please be respectful and professional in all interactions. We aim to maintain a welcoming and inclusive environment for all contributors.

## How to Contribute

### Reporting Issues

- Check if the issue already exists in the issue tracker
- Provide a clear description of the problem
- Include steps to reproduce the issue
- Add relevant logs, screenshots, or error messages
- Specify your environment (OS, Rust version, etc.)

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Update documentation as needed
7. Submit a pull request

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/aurelia.git
cd aurelia

# Add upstream remote
git remote add upstream https://github.com/tricorefile/aurelia.git

# Create a feature branch
git checkout -b feature/your-feature-name

# Install dependencies and build
cargo build --all

# Run tests
cargo test --all

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. All commit messages must be in **English**.

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (formatting, missing semi-colons, etc.)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Examples

```bash
# Feature
git commit -m "feat: add self-replication capability to autonomous agent"
git commit -m "feat(ssh): support password authentication for deployment"

# Bug fix
git commit -m "fix: resolve connection timeout in SSH deployer"
git commit -m "fix(monitoring): correct memory usage calculation"

# Documentation
git commit -m "docs: update deployment guide with new options"
git commit -m "docs(api): add WebSocket endpoint documentation"

# Style
git commit -m "style: format code according to rustfmt standards"

# Refactor
git commit -m "refactor: simplify decision-making algorithm"
git commit -m "refactor(kernel): extract event handling to separate module"

# Performance
git commit -m "perf: optimize market data processing pipeline"

# Test
git commit -m "test: add integration tests for deployment module"

# Build
git commit -m "build: update dependencies to latest versions"
git commit -m "build(docker): optimize container size"

# CI
git commit -m "ci: add automated release workflow"
git commit -m "ci(github): fix deprecated action versions"

# Chore
git commit -m "chore: update .gitignore"
git commit -m "chore(deps): bump tokio from 1.32 to 1.35"
```

### Commit Message Rules

1. **Use English only** - All commit messages must be in English
2. **Use present tense** - "add feature" not "added feature"
3. **Use imperative mood** - "fix bug" not "fixes bug" or "fixed bug"
4. **First line limited to 72 characters**
5. **Reference issues when applicable** - "fix: resolve login issue (#123)"
6. **Be descriptive but concise**
7. **Separate subject from body with a blank line**
8. **Explain what and why, not how** (the code shows how)

### Multi-line Commit Messages

For complex changes, use a multi-line commit message:

```bash
git commit -m "feat: implement distributed health monitoring system

- Add health check endpoints for all services
- Implement automatic failover mechanism
- Create dashboard for real-time monitoring
- Add alerting system for critical events

Closes #123, #124"
```

## Code Style

### Rust Code

- Follow Rust naming conventions
- Use `rustfmt` for formatting
- Pass `clippy` checks with no warnings
- Write descriptive variable and function names
- Add documentation comments for public APIs
- Keep functions small and focused

### Python Code

- Follow PEP 8 style guide
- Use type hints where applicable
- Add docstrings to functions and classes

### Shell Scripts

- Use `#!/bin/bash` shebang
- Set `set -e` for error handling
- Use meaningful variable names
- Add comments for complex logic

## Testing

### Test Requirements

- All new features must include tests
- Bug fixes should include regression tests
- Maintain or improve code coverage
- Tests must pass on all supported platforms

### Running Tests

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_test
```

## Documentation

- Update README.md for significant changes
- Add inline documentation for complex code
- Update API documentation for public interfaces
- Include examples for new features
- Keep documentation in sync with code

## Pull Request Process

1. **Ensure CI passes** - All checks must be green
2. **Update documentation** - README, API docs, etc.
3. **Add tests** - For new features and bug fixes
4. **Follow code style** - Use formatters and linters
5. **Write clear PR description** - Explain what and why
6. **Reference related issues** - Use "Closes #123" syntax
7. **Be responsive to feedback** - Address review comments promptly

### PR Title Format

PR titles should follow the same convention as commit messages:

- `feat: add new monitoring dashboard`
- `fix: resolve memory leak in agent`
- `docs: improve deployment instructions`

## Release Process

1. Releases are managed by maintainers
2. Follow semantic versioning (MAJOR.MINOR.PATCH)
3. Update CHANGELOG.md with release notes
4. Tag releases with `v` prefix (e.g., `v1.2.3`)

## Getting Help

- Check the documentation first
- Search existing issues
- Ask questions in issues with "question" label
- Join our Discord server (if applicable)

## Recognition

Contributors will be recognized in:
- The project README
- Release notes
- Special thanks section

Thank you for contributing to Aurelia!