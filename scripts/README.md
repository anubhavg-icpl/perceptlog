# PerceptLog Transform Scripts

This directory contains production-ready transformation scripts for converting various log formats to OCSF v1.6.0.

## Directory Structure

```
scripts/
├── README.md                           This file
├── production/                         Production scripts
│   └── linux_auth_ocsf.perceptlog     Linux authentication to OCSF
└── examples/                           Example scripts
    └── simple_transform.perceptlog     Simple OCSF transformation
```

## Production Scripts

### `production/linux_auth_ocsf.perceptlog`

**Production-ready Linux authentication log to OCSF transformation**

- **Lines**: ~870
- **OCSF Version**: 1.6.0
- **Source Format**: Linux syslog (auth.log, secure)
- **Target**: OCSF Authentication [3002]

**Features**:
- Parses SSH authentication events
- Handles sudo commands
- Processes PAM events
- Extracts user information
- Captures network endpoints
- Error handling and validation
- Observable extraction for threat detection

**Usage**:
```bash
perceptlog transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i /var/log/auth.log \
  -o output/ocsf.ndjson
```

**Supported Event Types**:
- SSH password authentication (success/failure)
- SSH key authentication
- Sudo command execution
- PAM session events
- User account changes
- System authentication failures
- Connection events

## Example Scripts

### `examples/simple_transform.perceptlog`

**Simple OCSF transformation template**

A minimal example showing OCSF structure and required fields.

**Usage**:
```bash
perceptlog transform \
  -s scripts/examples/simple_transform.perceptlog \
  -i input.log \
  -o output.json
```

## Writing Your Own Scripts

### Basic Structure

Every OCSF event requires:

```vrl
.category_uid = <number>       # OCSF category
.class_uid = <number>          # OCSF class
.time = <unix_timestamp>       # Event timestamp
.activity_id = <number>        # Activity type
.severity_id = <number>        # Severity level
.status_id = <number>          # Event status
.metadata = { ... }            # Required metadata
```

### Common Patterns

#### Parse Syslog Format
```vrl
parsed = parse_syslog!(.message)
.time = to_unix_timestamp(parsed.timestamp)
.hostname = parsed.hostname
```

#### Extract with Regex
```vrl
pattern = r'user=(?P<user>\S+)'
match = parse_regex(.message, pattern)
.user = { "name": match.user }
```

#### Conditional Logic
```vrl
if contains(.message, "Accepted") {
  .status = "Success"
  .status_id = 1
} else {
  .status = "Failure"
  .status_id = 2
}
```

## Testing Scripts

### Validate Script Syntax
```bash
perceptlog validate scripts/production/linux_auth_ocsf.perceptlog
```

### Test with Sample Data
```bash
perceptlog transform \
  -s scripts/production/linux_auth_ocsf.perceptlog \
  -i test_data/auth.log \
  -o output/test.json \
  -f json-pretty
```

### Run End-to-End Tests
```bash
cargo test e2e_ --test e2e_integration_tests
```

## Script Development Tips

1. **Start Simple**: Begin with required OCSF fields
2. **Test Incrementally**: Test each pattern as you add it
3. **Handle Errors**: Use error handling (! operator)
4. **Add Comments**: Document complex patterns
5. **Validate Output**: Check OCSF compliance

## OCSF Resources

- **Schema**: https://schema.ocsf.io/
- **Documentation**: https://github.com/ocsf/ocsf-docs
- **Examples**: https://github.com/ocsf/examples

## VRL Resources

- **Documentation**: https://vector.dev/docs/reference/vrl/
- **Functions**: https://vector.dev/docs/reference/vrl/functions/
- **Examples**: https://vector.dev/docs/reference/vrl/examples/

## Support

For issues or questions:
- Check test_data/ for example log files
- Review tests/e2e_integration_tests.rs for usage examples
- See main README.md for general documentation
