# QoreLogic A.E.G.I.S. Framework for Claude Code

A governance framework for AI-assisted development with cryptographic traceability, KISS enforcement, and persona-based routing.

## Overview

This adaptation brings the QoreLogic A.E.G.I.S. (Align, Encode, Gate, Implement, Substantiate) methodology to Claude Code, providing:

- **Macro-level KISS**: Project structure, file organization, dependency management
- **Micro-level KISS**: Function length, nesting depth, variable naming
- **Merkle-chained traceability**: Every decision cryptographically linked
- **Persona-based routing**: Right expertise for each domain

## Installation

### 1. Copy to Your Project

```bash
# Copy the bundle into your project-local Claude Code config
cp -r qorelogic/Claude/* /path/to/your/project/.claude/
```

### 2. Configure Claude Code Settings

Add to your project's `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": {
      "Write": ["hooks/kiss-razor-gate.json", "hooks/orphan-detection.json"],
      "Edit": ["hooks/kiss-razor-gate.json", "hooks/security-path-alert.json"]
    },
    "PostToolUse": {
      "Read": ["hooks/cognitive-reset.json"]
    },
    "Stop": ["hooks/session-seal.json"]
  }
}
```

### 3. Initialize QoreLogic DNA

```bash
# In Claude Code, run the bootstrap skill
/ql-bootstrap
```

## Directory Structure

```
.claude/
|-- README.md                 # This file
|-- commands/                # Slash command skills
|   |-- agents/              # QoreLogic agent personas
|   |   |-- ql-governor.md
|   |   |-- ql-judge.md
|   |   `-- ql-specialist.md
|   |-- references/          # Supporting patterns/templates
|   |-- scripts/             # Python utilities
|   |-- ql-bootstrap.md      # Initialize project DNA
|   |-- ql-status.md         # Lifecycle diagnostic
|   |-- ql-audit.md          # Gate tribunal (PASS/VETO)
|   |-- ql-implement.md      # Build with Section 4 Razor
|   |-- ql-refactor.md       # KISS simplification pass
|   |-- ql-validate.md       # Merkle chain verification
|   `-- ql-substantiate.md   # Session seal
|-- hooks/                   # Event hooks
|   |-- kiss-razor-gate.json     # Pre-tool KISS gate
|   |-- security-path-alert.json # Path-based security alert
|   |-- session-seal.json        # End-of-session seal
|   |-- cognitive-reset.json     # Post-read reset
|   `-- orphan-detection.json    # Orphan file detection
|-- agents/                  # General agent definitions
|-- docs/                    # Framework documentation
|-- improvements/            # Improvement tracking
|-- templates/               # A.E.G.I.S. document templates
|   |-- CONCEPT.md           # Strategic "Why" template
|   |-- ARCHITECTURE_PLAN.md # Technical blueprint template
|   |-- META_LEDGER.md       # Merkle chain template
|   |-- SYSTEM_STATE.md      # Project tree snapshot
|   `-- SHADOW_GENOME.md     # Failure mode documentation
`-- settings.json            # Claude Code settings
```

## The A.E.G.I.S. Lifecycle

```
+-----------------------------------------------------------------+
|                     A.E.G.I.S. LIFECYCLE                        |
|-----------------------------------------------------------------+
|                                                                 |
|  ALIGN --> ENCODE --> GATE --> IMPLEMENT --> SUBSTANTIATE      |
|    |         |         |          |              |              |
|    v         v         v          v              v              |
| CONCEPT  BLUEPRINT  VERDICT    CODE         SEALED            |
|   .md      .md      PASS/VETO   src/        LEDGER            |
|                                                                 |
|  [Governor]  [Governor]  [Judge]  [Specialist]  [Judge]        |
`-----------------------------------------------------------------+
```

### Phase Descriptions

| Phase | Persona | Purpose | Artifact |
|-------|---------|---------|----------|
| **ALIGN** | Governor | Define the "Why" in one sentence | `docs/CONCEPT.md` |
| **ENCODE** | Governor | Technical blueprint with risk grade | `docs/ARCHITECTURE_PLAN.md` |
| **GATE** | Judge | Adversarial audit, PASS/VETO verdict | `.agent/staging/AUDIT_REPORT.md` |
| **IMPLEMENT** | Specialist | Build with Section 4 Razor constraints | `src/*` |
| **SUBSTANTIATE** | Judge | Verify reality matches promise, seal | `docs/META_LEDGER.md` |

## KISS Enforcement (Section 4 Simplicity Razor)

### Macro Level (Project Structure)
- Files must not exceed **250 lines**
- No "God Objects" or utility dumping grounds
- Dependencies must prove necessity (10-line vanilla rule)
- All files must be in active build path (no orphans)

### Micro Level (Code Quality)
- Functions must not exceed **40 lines**
- Maximum **3 levels of nesting**
- No nested ternaries: `a - b : c - d : e`
- Explicit naming: `noun` or `verbNoun` (no `x`, `data`, `obj`)
- No `console.log` artifacts in production code

## Risk Grading System

| Grade | Definition | Examples | Verification |
|-------|------------|----------|--------------|
| **L1** | Routine, reversible | UI text, comments, renames | Static analysis |
| **L2** | Logic changes | New functions, APIs, schemas | Sentinel + Citation |
| **L3** | Security/irreversible | Auth, encryption, PII, keys | Formal review + seal |

## Quick Start Commands

Note: Slash commands like `/ql-*` are available in the Claude Code CLI (`claude`), not the VS Code extension UI. If you are using only the extension, you will not see these commands.

```bash
# Initialize a new project with QoreLogic DNA
/ql-bootstrap

# Check project lifecycle status
/ql-status

# Run gate tribunal for L2/L3 changes
/ql-audit

# Implement with KISS razor enforcement
/ql-implement

# Refactor existing code for KISS compliance
/ql-refactor

# Validate Merkle chain integrity
/ql-validate

# Seal session and verify promise matches reality
/ql-substantiate
```

## Automatic Persona Routing

Claude Code does not currently spawn custom subagents from project settings. The persona files are guidance, not isolated agents. Route manually using the slash commands below, or use Claude Code's built-in Task tool agents when you need true isolation.

To ensure persona context is loaded, each `/ql-*` skill now reads a matching persona skill file (for example, `.claude/commands/ql-judge-persona.md`).

When routing manually, use these path-based guidelines:

| Path Pattern | Routed Persona | Enforcement |
|--------------|----------------|-------------|
| `*/security/*`, `*/auth/*` | Judge | L3 lockdown, formal review |
| `*/src/*`, `*/components/*` | Specialist | 40-line razor |
| `*/docs/*` | Governor | Merkle chain verification |

## Merkle Chain Traceability

Every significant decision is cryptographically linked:

```
Entry #1 (Genesis)
  hash: SHA256(CONCEPT.md + ARCHITECTURE_PLAN.md)

Entry #2 (Audit)
  hash: SHA256(AUDIT_REPORT.md + Entry#1.hash)

Entry #3 (Implementation)
  hash: SHA256(src/* changes + Entry#2.hash)

Entry #4 (Seal)
  hash: SHA256(final state + Entry#3.hash)
```

## Migration from Zo

If you have an existing Zo configuration:

1. The persona definitions map directly to subagents
2. The prompts become skills (slash commands)
3. The JSON rules become hooks + skill enforcement
4. Document templates remain compatible

## Troubleshooting

### "Gate locked. Tribunal audit required."
Run `/ql-audit` to generate a PASS/VETO verdict before implementation.

### "Chain Broken at Entry #X"
The Merkle chain has been tampered with or corrupted. Manual audit required.

### "Complexity razor triggered"
Your code exceeds KISS constraints. Run `/ql-refactor` for automatic simplification.

### "Target file appears orphaned"
The file is not connected to the build path. Verify imports or remove dead code.
