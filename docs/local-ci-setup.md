# Local CI/CD Setup Guide

This guide explains how to set up and use local CI/CD tools that mirror the GitHub Actions pipeline, allowing you to catch issues before pushing to GitHub.

## Quick Setup

```bash
# One-command setup
./scripts/setup-local-ci.sh

# Or step by step
make setup-dev
```

## Available Scripts

### 1. Full Local CI Pipeline

Mirrors the complete GitHub Actions workflow:

```bash
# Run the complete CI pipeline locally
make ci-local
# or
./scripts/local-ci.sh
```

**What it does:**
- ✅ Code formatting check (`cargo fmt --all -- --check`)
- ✅ Clippy linting (`cargo clippy --workspace --all-targets --all-features -- -D warnings`)
- ✅ Workspace build (`cargo build --workspace`)
- ✅ Unit tests (`cargo test --workspace --lib`)
- ✅ Integration tests (`cargo test --workspace --test '*'`)
- ✅ Platform-specific tests (Linux/macOS)
- ✅ Steel test suite
- ✅ Steel examples
- ✅ Security audit (`cargo audit`)
- ✅ Supply chain security (`cargo deny`)
- ✅ Documentation build
- ✅ Steel program validation

### 2. Pre-Push Checks

Essential checks before pushing to GitHub:

```bash
# Full pre-push validation
make pre-push
# or
./scripts/pre-push.sh

# Quick pre-push checks (faster)
make pre-push-quick
# or
./scripts/pre-push.sh --quick

# Skip specific checks
./scripts/pre-push.sh --skip-tests    # Skip tests
./scripts/pre-push.sh --skip-esp32    # Skip ESP32 checks
```

**What it does:**
- ✅ Auto-fixes code formatting
- ✅ Clippy linting
- ✅ Build check
- ✅ Unit and integration tests (unless skipped)
- ✅ ESP32 cross-compilation check (unless skipped)
- ✅ Security audit

### 3. ESP32 Cross-Compilation Test

Tests ESP32 compilation without requiring full ESP-IDF setup:

```bash
make test-esp32
# or
./scripts/test-esp32-build.sh
```

**What it does:**
- ✅ Checks ESP32 target installation
- ✅ Validates ESP32 package structure
- ✅ Tests cross-compilation (if ESP-IDF available)
- ✅ Builds release binary
- ✅ Reports binary sizes

## Git Hooks

The setup script automatically installs Git hooks:

### Pre-Commit Hook
Runs automatically before each commit:
- Quick formatting and linting checks
- Prevents commits with obvious issues
- Takes ~30 seconds

### Pre-Push Hook
Runs automatically before each push:
- Full pre-push validation
- Prevents pushing broken code
- Takes ~2-5 minutes

**To bypass hooks (not recommended):**
```bash
git commit --no-verify    # Skip pre-commit
git push --no-verify      # Skip pre-push
```

## Comparison with GitHub Actions

| Check | Local CI | GitHub Actions | Notes |
|-------|----------|----------------|-------|
| Code formatting | ✅ | ✅ | Identical |
| Clippy linting | ✅ | ✅ | Identical |
| Unit tests | ✅ | ✅ | Identical |
| Integration tests | ✅ | ✅ | Identical |
| ESP32 cross-compilation | ⚠️ | ✅ | Requires ESP-IDF setup locally |
| Security audit | ✅ | ✅ | Identical |
| Steel validation | ✅ | ✅ | Identical |
| Firmware signing | ❌ | ✅ | CI-only (requires secrets) |
| AWS deployment | ❌ | ✅ | CI-only (requires credentials) |

## Development Workflow

### Recommended Workflow

1. **Setup** (one time):
   ```bash
   ./scripts/setup-local-ci.sh
   ```

2. **Development cycle**:
   ```bash
   # Make changes
   vim src/main.rs
   
   # Quick check (auto-runs via Git hook)
   git commit -m "Add feature"
   
   # Full validation before push
   make pre-push
   
   # Push (auto-runs via Git hook)
   git push
   ```

3. **Before major changes**:
   ```bash
   # Run full CI locally
   make ci-local
   ```

### Time Comparison

| Command | Time | Use Case |
|---------|------|----------|
| `make pre-push-quick` | ~30s | Quick commit validation |
| `make pre-push` | ~2-5min | Before pushing |
| `make ci-local` | ~5-10min | Before major changes |
| GitHub Actions | ~10-15min | Automatic on push |

## Troubleshooting

### Common Issues

**0. Rust version too old (icu_normalizer error):**
```bash
# Quick fix
make fix-rust
# or manually
rustup update stable
cargo clean && cargo update
```

**1. ESP32 compilation fails (macOS):**
```bash
# Quick setup for macOS
make setup-esp32

# Or manual setup
cargo install espup
espup install
source $HOME/export-esp.sh

# Note: ESP32 target isn't available in standard Rust on macOS
# It requires the ESP-IDF toolchain installed via espup
```

**2. Security audit fails:**
```bash
# Install cargo-audit
cargo install cargo-audit

# Update advisory database
cargo audit --update
```

**3. Clippy warnings:**
```bash
# Auto-fix where possible
cargo clippy --workspace --all-targets --fix

# Allow specific warnings (in code)
#[allow(clippy::warning_name)]
```

**4. Formatting issues:**
```bash
# Auto-fix formatting
cargo fmt --all
```

### Performance Tips

**Speed up local CI:**
- Use `--quick` mode for rapid feedback
- Skip tests during development: `--skip-tests`
- Skip ESP32 checks: `--skip-esp32`
- Use `cargo check` instead of `cargo build` for syntax checking

**Parallel execution:**
```bash
# Run tests in parallel
cargo test --workspace -- --test-threads=4

# Build in parallel
cargo build --workspace -j 4
```

## Configuration

### Customizing Checks

Edit the scripts to customize behavior:

- `scripts/local-ci.sh` - Full CI pipeline
- `scripts/pre-push.sh` - Pre-push checks
- `scripts/test-esp32-build.sh` - ESP32 testing

### Security Policies

Edit `deny.toml` to customize security policies:
- Allowed/denied licenses
- Security vulnerability handling
- Dependency policies

### Git Hooks

Hooks are in `.git/hooks/`:
- `pre-commit` - Quick checks
- `pre-push` - Full validation

## Integration with IDEs

### VS Code

Add to `.vscode/tasks.json`:
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Local CI",
            "type": "shell",
            "command": "make",
            "args": ["ci-local"],
            "group": "test",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        },
        {
            "label": "Pre-push Check",
            "type": "shell",
            "command": "make",
            "args": ["pre-push"],
            "group": "test"
        }
    ]
}
```

### IntelliJ/CLion

Add run configurations for:
- `make ci-local`
- `make pre-push`
- `make test-esp32`

## Continuous Integration Benefits

**Faster feedback:**
- Catch issues in ~2 minutes vs ~15 minutes in CI
- Fix problems before they reach GitHub
- Reduce CI pipeline failures

**Cost savings:**
- Fewer CI minutes used
- Less compute time on GitHub Actions
- Faster development cycles

**Better code quality:**
- Consistent formatting and linting
- Security vulnerability detection
- Dependency policy enforcement

**Team productivity:**
- Standardized development environment
- Automated quality checks
- Reduced review overhead