# PerceptLog Quick Start Guide

Get started with PerceptLog in 5 minutes!

## 🚀 Quick Commands

### 1. Run All Tests
```bash
./run_e2e_tests.sh
```

### 2. Transform Linux Auth Logs
```bash
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/result.ndjson
```

### 3. Validate Script
```bash
cargo run -- validate scripts/production/linux_auth_ocsf.perceptlog
```

### 4. View Output
```bash
cat output/result.ndjson | head -1 | jq .
```

## 📁 Project Structure

```
perceptlog/
├── scripts/
│   ├── production/           Production-ready scripts
│   │   └── linux_auth_ocsf.perceptlog  (869 lines)
│   └── examples/             Example scripts
│       └── simple_transform.perceptlog
├── test_data/               Real log data
│   ├── auth.log             Debian logs (12,000 lines)
│   ├── secure               RHEL logs (600 lines)
│   └── sample_auth.log      Samples (5 lines)
├── tests/                   Test suite
│   └── e2e_integration_tests.rs
└── src/                     Source code
    ├── cli/                 CLI interface
    ├── core/                Core types
    ├── processing/          VRL runtime
    ├── io/                  File operations
    ├── output/              Output formatting
    └── utils/               Utilities
```

## 🧪 Testing

### Run E2E Tests
```bash
# All end-to-end tests
cargo test e2e_ --test e2e_integration_tests

# Specific test
cargo test test_e2e_ssh_patterns --test e2e_integration_tests -- --nocapture

# With test runner
./run_e2e_tests.sh
```

### Run Unit Tests
```bash
# All tests
cargo test

# VRL runtime tests only
cargo test --lib processing::runtime
```

## 📊 Usage Examples

### Basic Transformation
```bash
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i /var/log/auth.log \
  -o output/ocsf.ndjson
```

### With Pretty JSON
```bash
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/sample_auth.log \
  -o output/result.json \
  -f json-pretty
```

### Multiple Formats
```bash
# NDJSON (streaming)
cargo run -- transform -s <script> -i <input> -o out.ndjson -f ndjson

# JSON (compact)
cargo run -- transform -s <script> -i <input> -o out.json -f json

# JSON Pretty (readable)
cargo run -- transform -s <script> -i <input> -o out.json -f json-pretty

# YAML
cargo run -- transform -s <script> -i <input> -o out.yaml -f yaml
```

## 🔍 Inspecting Results

### View Transformed Output
```bash
# First event
cat output/result.ndjson | head -1 | jq .

# All events (pretty)
cat output/result.json | jq .

# Extract specific fields
cat output/result.ndjson | jq '{category_uid, class_uid, activity_name, user}'
```

### Count Events
```bash
wc -l output/result.ndjson
```

### Extract Unique Values
```bash
# Unique users
cat output/result.ndjson | jq -r '.user.name' | sort -u

# Unique source IPs
cat output/result.ndjson | jq -r '.src_endpoint.ip' | sort -u

# Activity summary
cat output/result.ndjson | jq -r '.activity_name' | sort | uniq -c
```

## 🛠️ Development

### Build
```bash
cargo build
cargo build --release  # Optimized
```

### Lint
```bash
cargo clippy --lib -- -D warnings
```

### Format
```bash
cargo fmt
```

### Clean
```bash
cargo clean
```

## 📖 Available Scripts

### Production Scripts

| Script | Description | Lines | Source |
|--------|-------------|-------|--------|
| `linux_auth_ocsf.perceptlog` | Linux auth → OCSF | 869 | Debian/RHEL |

### Example Scripts

| Script | Description | Lines | Purpose |
|--------|-------------|-------|---------|
| `simple_transform.perceptlog` | Basic OCSF | 39 | Learning |

## 🎯 Common Tasks

### Test New Script
```bash
# 1. Create script
nano scripts/examples/my_script.perceptlog

# 2. Validate
cargo run -- validate scripts/examples/my_script.perceptlog

# 3. Test
cargo run -- transform \
  -s scripts/examples/my_script.perceptlog \
  -i test_data/sample_auth.log \
  -o output/test.json \
  -f json-pretty

# 4. View result
cat output/test.json | jq .
```

### Process Real Logs
```bash
# Transform your logs
cargo run -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i /var/log/auth.log \
  -o ~/ocsf_output.ndjson

# Check output
cat ~/ocsf_output.ndjson | wc -l
cat ~/ocsf_output.ndjson | head -1 | jq .
```

### Benchmark Performance
```bash
time cargo run --release -- transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/benchmark.ndjson

# Expected: ~10,000+ events/second
```

## 🐛 Troubleshooting

### Script Errors
```bash
# Validate syntax
cargo run -- validate scripts/production/linux_auth_ocsf.perceptlog

# Check for specific error
cargo run -- transform -s <script> -i <input> -o <output> 2>&1 | grep -i error
```

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build

# Check dependencies
cargo tree
```

### Test Failures
```bash
# Run specific test with output
cargo test test_e2e_ssh_patterns --test e2e_integration_tests -- --nocapture

# Check test data
ls -la test_data/
```

## 📚 Documentation

- **Full Documentation**: See [README.md](README.md)
- **E2E Testing**: See [RUNNING_E2E_TESTS.md](RUNNING_E2E_TESTS.md)
- **Script Writing**: See [scripts/README.md](scripts/README.md)
- **VRL Compliance**: See [VRL_COMPLIANCE.md](VRL_COMPLIANCE.md)

## 🎓 Learning Path

1. **Start Here**: Run `./run_e2e_tests.sh`
2. **Explore**: Check `scripts/examples/simple_transform.perceptlog`
3. **Test**: Run `cargo test test_e2e_ssh_patterns -- --nocapture`
4. **Transform**: Try with `test_data/sample_auth.log`
5. **Inspect**: View output with `jq`
6. **Create**: Write your own script

## ✅ Verification Checklist

- [ ] Tests run: `./run_e2e_tests.sh`
- [ ] Build works: `cargo build`
- [ ] Script validates: `cargo run -- validate <script>`
- [ ] Transform works: `cargo run -- transform ...`
- [ ] Output valid: `cat output/*.ndjson | head -1 | jq .`

## 🚀 Next Steps

After getting started:

1. Review production script in `scripts/production/`
2. Explore test data in `test_data/`
3. Read E2E testing guide: `RUNNING_E2E_TESTS.md`
4. Check VRL compliance: `VRL_COMPLIANCE.md`
5. Write your own transformation scripts

## 💡 Tips

- Use `--nocapture` with tests to see output
- Use `-f json-pretty` for readable output
- Use `jq` to inspect JSON output
- Check `scripts/README.md` for writing tips
- Run `./run_e2e_tests.sh` to verify everything

---

**Status**: ✅ Production Ready  
**VRL Compliance**: 100%  
**Tests**: 52+ passing  
**Production Script**: 869 lines
