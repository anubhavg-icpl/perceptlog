# Running End-to-End Tests

Complete guide for running PerceptLog end-to-end tests with real production data.

## Prerequisites

```bash
# Ensure you're in the project directory
cd /Users/anubhavg/Desktop/perceptlog

# Verify structure
ls -la scripts/ test_data/ tests/
```

## Quick Start

### 1. Run All E2E Tests
```bash
cargo test e2e_ --test e2e_integration_tests -- --nocapture
```

### 2. Run Specific E2E Test
```bash
# Test SSH pattern recognition
cargo test test_e2e_ssh_patterns --test e2e_integration_tests -- --nocapture

# Test sudo patterns
cargo test test_e2e_sudo_patterns --test e2e_integration_tests -- --nocapture

# Test batch processing
cargo test test_e2e_batch_processing --test e2e_integration_tests -- --nocapture
```

### 3. Run with Production Script
```bash
# Test with real Linux auth logs
cargo test test_e2e_debian_auth_log --test e2e_integration_tests -- --nocapture

# Test with RHEL logs
cargo test test_e2e_rhel_secure_log --test e2e_integration_tests -- --nocapture
```

## Manual Testing

### Test Production Script Directly

```bash
# Test with Debian logs
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/debian_ocsf.ndjson \
  -f ndjson

# Test with RHEL logs
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/secure \
  -o output/rhel_ocsf.json \
  -f json-pretty
```

### Test Simple Example Script

```bash
# Use example script
cargo run -- transform \
  -s scripts/examples/simple_transform.perceptlog \
  -i test_data/sample_auth.log \
  -o output/simple.json \
  -f json-pretty
```

### Validate Scripts

```bash
# Validate production script
cargo run -- validate scripts/production/linux_auth_ocsf.perceptlog

# Validate example script
cargo run -- validate scripts/examples/simple_transform.perceptlog
```

## Test Data

### Available Test Files

```
test_data/
├── auth.log              # Debian/Ubuntu auth logs (12,000 lines)
├── secure                # RHEL/CentOS secure logs (600 lines)
└── sample_auth.log       # Curated test samples (5 lines)
```

### View Test Data

```bash
# View Debian logs
head -20 test_data/auth.log

# View RHEL logs
head -20 test_data/secure

# View samples
cat test_data/sample_auth.log
```

## Complete Test Suite

### 1. Unit Tests (VRL Runtime)
```bash
# Test VRL runtime functionality
cargo test --lib processing::runtime -- --nocapture

# Expected output:
# test processing::runtime::tests::test_vrl_runtime_simple ... ok
# test processing::runtime::tests::test_vrl_runtime_field_access ... ok
# test processing::runtime::tests::test_value_conversions ... ok
# test processing::runtime::tests::test_log_event_conversion ... ok
```

### 2. Integration Tests (E2E)
```bash
# Run all end-to-end tests
cargo test --test e2e_integration_tests -- --nocapture

# Expected tests:
# - test_e2e_with_sample_logs
# - test_e2e_debian_auth_log
# - test_e2e_rhel_secure_log
# - test_e2e_ssh_patterns
# - test_e2e_sudo_patterns
# - test_e2e_batch_processing
# - test_e2e_multiple_formats
# - test_e2e_ocsf_compliance
```

### 3. All Tests
```bash
# Run complete test suite
cargo test -- --nocapture

# Run in release mode (faster)
cargo test --release
```

## Output Formats

Test with different output formats:

```bash
# NDJSON (streaming)
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/result.ndjson \
  -f ndjson

# JSON (compact)
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/result.json \
  -f json

# JSON Pretty (readable)
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/result.json \
  -f json-pretty

# YAML
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/result.yaml \
  -f yaml
```

## Verify Output

### Check OCSF Compliance

```bash
# View transformed output
cat output/debian_ocsf.ndjson | head -1 | jq .

# Verify required fields
cat output/debian_ocsf.ndjson | jq '{
  category_uid,
  class_uid,
  time,
  activity_id,
  severity_id,
  status_id
}'

# Count events
wc -l output/debian_ocsf.ndjson
```

### Check Specific Fields

```bash
# Extract users
cat output/debian_ocsf.ndjson | jq -r '.user.name' | sort -u

# Extract source IPs
cat output/debian_ocsf.ndjson | jq -r '.src_endpoint.ip' | sort -u

# Extract activities
cat output/debian_ocsf.ndjson | jq -r '.activity_name' | sort | uniq -c
```

## Performance Testing

### Measure Throughput

```bash
# Time the transformation
time cargo run --release -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/perf_test.ndjson

# Expected: ~10,000+ events/second
```

### Batch Processing

```bash
# Process with specific batch size
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/batched.ndjson \
  --batch-size 1000
```

## Debugging

### Enable Debug Logging

```bash
# Set log level
export RUST_LOG=perceptlog=debug

# Run with debug output
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/sample_auth.log \
  -o output/debug.json \
  -f json-pretty
```

### Handle Errors

```bash
# Skip errors and continue
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/result.ndjson \
  --skip-errors

# Save errors to file
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/result.ndjson \
  --error-output errors.log
```

## CI/CD Integration

### GitHub Actions

```yaml
name: E2E Tests
on: [push, pull_request]
jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run E2E tests
        run: cargo test --test e2e_integration_tests
```

### Docker

```bash
# Build image
docker build -t perceptlog .

# Run tests in container
docker run --rm perceptlog cargo test --test e2e_integration_tests
```

## Troubleshooting

### Test Failures

If tests fail:

1. **Check VRL script syntax**:
   ```bash
   cargo run -- validate scripts/production/linux_auth_ocsf.perceptlog
   ```

2. **Verify test data exists**:
   ```bash
   ls -lh test_data/
   ```

3. **Check dependencies**:
   ```bash
   cargo build --lib
   ```

4. **Run specific test**:
   ```bash
   cargo test test_e2e_ssh_patterns --test e2e_integration_tests -- --nocapture
   ```

### Common Issues

**Issue**: "Script file not found"
```bash
# Solution: Check script path
ls -la scripts/production/linux_auth_ocsf.perceptlog
```

**Issue**: "No test data"
```bash
# Solution: Verify test data
ls -la test_data/
```

**Issue**: "VRL compilation error"
```bash
# Solution: Validate script
cargo run -- validate scripts/production/linux_auth_ocsf.perceptlog
```

## Expected Test Output

### Successful Run

```
running 8 tests
✓ Read 12000 events from Debian auth.log
✓ Transformed 12000 events to OCSF format
✓ Wrote OCSF events to /tmp/.../debian_ocsf.ndjson
✓ End-to-end Debian auth.log transformation successful

test test_e2e_ssh_patterns ... ok
test test_e2e_sudo_patterns ... ok
test test_e2e_batch_processing ... ok
test test_e2e_multiple_formats ... ok

test result: ok. 8 passed; 0 failed
```

## Summary

### Quick Commands Reference

```bash
# Run all E2E tests
cargo test e2e_ --test e2e_integration_tests

# Transform with production script
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/result.ndjson

# Validate script
cargo run -- validate scripts/production/linux_auth_ocsf.perceptlog

# View output
cat output/result.ndjson | head -1 | jq .

# Run all tests
cargo test
```

### Test Coverage

- ✅ VRL Runtime: 4 tests
- ✅ E2E Integration: 8 tests
- ✅ Real Data: 12,605 log lines
- ✅ Production Scripts: 869 lines
- ✅ All Formats: JSON, NDJSON, YAML

**Status**: All infrastructure in place and working ✅
