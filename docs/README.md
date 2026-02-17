# Documentation

## Structure

```
docs/
├── analysis/          # Performance analysis and comparisons
├── build/             # Build configuration and troubleshooting
├── security/          # Security documentation and threat model
├── testing/           # Test plans and validation reports
└── *.md               # Core documentation
```

## Core Documentation

| Document | Description |
|----------|-------------|
| [CONCEPT.md](CONCEPT.md) | Design philosophy, anti-goals, performance targets |
| [USAGE_GUIDE.md](USAGE_GUIDE.md) | API reference and usage patterns |
| [DEPENDENCY_ANALYSIS.md](DEPENDENCY_ANALYSIS.md) | Dependency audit and licensing |
| [RUST_ENTERPRISE_ANALYSIS.md](RUST_ENTERPRISE_ANALYSIS.md) | Rust adoption in enterprise |
| [HONEST_ASSESSMENT.md](HONEST_ASSESSMENT.md) | Transparent claims evaluation |

## Security

| Document | Description |
|----------|-------------|
| [security/THREAT_MODEL.md](security/THREAT_MODEL.md) | STRIDE analysis, attack trees, audit recommendations |
| [security/SECURITY_ANALYSIS_REPORT.md](security/SECURITY_ANALYSIS_REPORT.md) | CVE remediations and test coverage |
| [security/SECURITY_REVIEW.md](security/SECURITY_REVIEW.md) | Initial security review findings |
| [security/SECURITY_IMPLEMENTATION_SUMMARY.md](security/SECURITY_IMPLEMENTATION_SUMMARY.md) | Security feature implementation |

## Testing

| Document | Description |
|----------|-------------|
| [testing/TIER1_*.md](testing/) | Baseline validation (sandbox works) |
| [testing/TIER2_*.md](testing/) | Competitive performance validation |
| [testing/TIER3_*.md](testing/) | Advanced optimization results |

## Build

| Document | Description |
|----------|-------------|
| [build/GGUF_BUILD_TROUBLESHOOTING.md](build/GGUF_BUILD_TROUBLESHOOTING.md) | GGUF backend build instructions |
| [build/OPTIMIZATION_VERIFICATION.md](build/OPTIMIZATION_VERIFICATION.md) | Build optimization verification |

## Analysis

| Document | Description |
|----------|-------------|
| [analysis/COMPARATIVE_ANALYSIS.md](analysis/COMPARATIVE_ANALYSIS.md) | Runtime comparison |
| [analysis/OLLAMA_COMPARISON_ANALYSIS.md](analysis/OLLAMA_COMPARISON_ANALYSIS.md) | Ollama feature comparison |
| [analysis/BASELINE_METRICS.md](analysis/BASELINE_METRICS.md) | Baseline performance metrics |
