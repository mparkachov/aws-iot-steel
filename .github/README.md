# GitHub Actions CI/CD Pipeline

This directory contains the GitHub Actions workflows for the AWS IoT Steel project. The CI/CD pipeline is designed as a hybrid approach where GitHub Actions handles Rust compilation and testing, while AWS CodePipeline/CodeBuild handles infrastructure deployment and AWS-specific operations.

## Workflows

### 1. CI/CD Pipeline (`ci.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` branch

**Jobs:**
- **code-quality**: Runs rustfmt, clippy, and security audit
- **test-macos**: Runs comprehensive test suite on macOS (development platform)
- **cross-compile-esp32**: Cross-compiles for ESP32-C3-DevKit-RUST-1 target
- **build-and-sign**: Creates signed firmware packages (main/develop only)
- **upload-to-aws**: Uploads artifacts to AWS S3 and triggers CodePipeline (main only)

**Artifacts:**
- ESP32 firmware binaries
- Signed firmware packages
- Test results and coverage reports

### 2. Steel Programs CI/CD (`steel-programs.yml`)

**Triggers:**
- Changes to Steel program files (`aws-iot-core/examples/steel/**`, `steel-programs/**`)
- Push to `main` or `develop` branches
- Pull requests to `main` branch

**Jobs:**
- **validate-steel-programs**: Validates Steel program syntax and execution
- **package-steel-programs**: Creates Steel program packages with metadata
- **upload-steel-programs-to-aws**: Uploads Steel programs to AWS S3 (main only)

**Artifacts:**
- Validated Steel programs
- Steel program packages with metadata

### 3. Security and Dependency Monitoring (`security.yml`)

**Triggers:**
- Daily schedule (2 AM UTC)
- Changes to dependency files (`Cargo.toml`, `Cargo.lock`)
- Manual workflow dispatch

**Jobs:**
- **security-audit**: Runs `cargo audit` for known vulnerabilities
- **dependency-check**: Validates dependency licenses
- **outdated-dependencies**: Checks for outdated dependencies
- **supply-chain-security**: Runs `cargo deny` for supply chain security
- **notify-security-issues**: Creates notifications for security issues

**Artifacts:**
- Security audit reports
- License compliance reports
- Outdated dependency reports
- Supply chain security reports

## Required Secrets

The following secrets must be configured in the GitHub repository:

### AWS Integration
- `AWS_GITHUB_ACTIONS_ROLE_ARN`: ARN of the IAM role for GitHub Actions OIDC
- `AWS_REGION`: AWS region for deployments (e.g., `us-west-2`)
- `S3_BUILD_ARTIFACTS_BUCKET`: S3 bucket name for build artifacts

### Optional Secrets
- `FIRMWARE_SIGNING_KEY`: Private key for firmware signing (base64 encoded)
- `SLACK_WEBHOOK_URL`: Slack webhook for notifications
- `TEAMS_WEBHOOK_URL`: Microsoft Teams webhook for notifications

## AWS OIDC Setup

The pipeline uses OpenID Connect (OIDC) for secure authentication with AWS without storing long-lived credentials.

### 1. Create OIDC Identity Provider

```bash
aws iam create-open-id-connect-provider \
  --url https://token.actions.githubusercontent.com \
  --client-id-list sts.amazonaws.com \
  --thumbprint-list 6938fd4d98bab03faadb97b34396831e3780aea1
```

### 2. Create IAM Role

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::ACCOUNT-ID:oidc-provider/token.actions.githubusercontent.com"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
        },
        "StringLike": {
          "token.actions.githubusercontent.com:sub": "repo:YOUR-ORG/aws-iot-steel:ref:refs/heads/main"
        }
      }
    }
  ]
}
```

### 3. Attach Minimal IAM Policy

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:PutObjectAcl"
      ],
      "Resource": [
        "arn:aws:s3:::your-build-artifacts-bucket/*"
      ]
    },
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::your-build-artifacts-bucket"
      ]
    }
  ]
}
```

## Workflow Customization

### Environment-Specific Configuration

Create environment-specific configurations by modifying the workflow files:

```yaml
# For staging environment
- name: Upload to staging
  if: github.ref == 'refs/heads/develop'
  run: |
    aws s3 cp artifacts/ s3://staging-build-artifacts-bucket/ --recursive

# For production environment  
- name: Upload to production
  if: github.ref == 'refs/heads/main'
  run: |
    aws s3 cp artifacts/ s3://prod-build-artifacts-bucket/ --recursive
```

### Adding New Targets

To add support for additional compilation targets:

```yaml
- name: Install additional targets
  run: |
    rustup target add aarch64-unknown-linux-gnu
    rustup target add x86_64-pc-windows-gnu

- name: Cross-compile for additional targets
  run: |
    cargo build --target aarch64-unknown-linux-gnu
    cargo build --target x86_64-pc-windows-gnu
```

### Custom Steel Program Validation

Extend Steel program validation by modifying the validation job:

```yaml
- name: Custom Steel validation
  run: |
    # Add custom validation rules
    cargo run --bin steel_program_validator -- \
      --file "$steel_file" \
      --validate-only \
      --custom-rules validation-rules.json
```

## Monitoring and Debugging

### Workflow Status

Monitor workflow status through:
- GitHub Actions tab in the repository
- GitHub CLI: `gh run list`
- GitHub API: `GET /repos/{owner}/{repo}/actions/runs`

### Debugging Failed Workflows

1. **Check workflow logs**: Click on failed job in GitHub Actions UI
2. **Download artifacts**: Use `gh run download` or GitHub UI
3. **Re-run failed jobs**: Use "Re-run failed jobs" button
4. **Enable debug logging**: Add `ACTIONS_STEP_DEBUG: true` to workflow

### Common Issues

1. **Cross-compilation failures**: Check ESP-IDF installation and target configuration
2. **AWS authentication errors**: Verify OIDC setup and IAM role permissions
3. **Steel program validation errors**: Check Steel syntax and runtime dependencies
4. **Artifact upload failures**: Verify S3 bucket permissions and network connectivity

## Security Considerations

1. **Secrets Management**: Never commit secrets to the repository
2. **OIDC Configuration**: Use specific repository and branch conditions
3. **Minimal Permissions**: Grant only necessary AWS permissions
4. **Artifact Scanning**: All artifacts are scanned for vulnerabilities
5. **Supply Chain Security**: Dependencies are regularly audited

## Integration with AWS CodePipeline

The GitHub Actions pipeline integrates with AWS CodePipeline through S3 triggers:

1. **GitHub Actions** builds and uploads artifacts to S3
2. **S3 Event** triggers AWS CodePipeline
3. **CodePipeline** orchestrates AWS-side deployment
4. **CodeBuild** handles infrastructure deployment and IoT configuration

This hybrid approach leverages the strengths of both platforms:
- **GitHub Actions**: Excellent for Rust compilation and testing
- **AWS CodePipeline/CodeBuild**: Native AWS integration and infrastructure management