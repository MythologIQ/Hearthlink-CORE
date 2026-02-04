Below is a complete, handoff-ready Architectural Design Document for the
Hearthlink CORE Runtime (Local Inference Runtime).

This is written intentionally like something you could drop directly into:

/docs/architecture/CORE_RUNTIME_ARCHITECTURE.md

It is:

precise

implementation-ready

security-first

not marketing

scoped only to the runtime (not Hearthlink overall)

aligned with your 4 pillars and earlier decisions

No fluff. Only engineering truth.

Hearthlink CORE Runtime

Contained • Offline • Restricted • Execution

Local Inference Runtime Architecture
Version 1.0

1. Purpose

The CORE Runtime is Hearthlink’s local, secure inference engine.

It provides deterministic model execution while enforcing strict isolation from:

system data

external networks

plugins

governance logic

memory stores

It exists solely to:

convert tokens to tokens safely and locally.

The runtime performs compute only.
All policy, permissions, and data ownership remain outside this component.

2. Design Principles

The CORE Runtime follows four non-negotiable constraints.

Contained

Runs inside a sandbox with no ambient privileges.

Offline

No inbound or outbound network access.

Restricted

Only accepts requests from Hearthlink through authenticated IPC.

Execution

Performs inference only. No business logic or decision authority.

If the runtime ever performs anything beyond execution, it violates its design.

3. Scope
Responsibilities (ONLY these)

Load models

Tokenize inputs

Run inference

Stream tokens

Manage GPU/CPU memory

Swap models

Schedule concurrent inference sessions

Return outputs

Explicitly Out of Scope

The runtime must NOT:

read Vault

write files outside temp/model dirs

access Synapse

call plugins

make HTTP requests

manage permissions

perform governance

orchestrate workflows

store persistent memory

Those belong to other modules.

4. Trust Model
Trust Assumptions

The runtime is:

trusted for computation

not trusted for authority

It is treated as:

high complexity

third-party heavy

difficult to audit

Therefore:

it receives minimal privileges.

Not because it is hostile.
Because its blast radius must be small.

5. System Placement
Logical Architecture

Hearthlink Stack

Control (governance)
Vault (data)
Construct (persona lifecycle)
Synapse (external boundaries)
?
CORE Runtime (compute only)

Data Flow

Control ? IPC ? CORE ? tokens ? Control

No other paths exist.

No direct file access.
No direct network access.
No direct Vault access.

6. Process Architecture
Runtime runs as

Separate OS process
Separate user
Lowest privileges possible

Never embedded in the main process.

Communication

Frontend/Backend ? Named IPC only

Allowed:

Unix domain sockets (mac/linux)

Named pipes (Windows)

Tauri invoke bridge

Forbidden:

HTTP

REST

localhost ports

WebSockets

Why:
Ports create hidden trust boundaries and attack surfaces.

7. Filesystem Policy
Read

models/
tokenizers/

Write

temp/
cache/

Deny

everything else

No access to:

Vault DB

user home

system directories

plugin folders

8. Network Policy

Outbound: blocked
Inbound: none

Firewall rules:

deny all

Rationale:

Local inference must never:

phone home

download code

leak prompts

fetch remote tools

All external traffic must route through Synapse only.

9. Runtime Capabilities
Model Management

load/unload models

hot swap models

multiple models concurrently

quantized variants

CPU/GPU selection

Scheduling

concurrent requests

streaming responses

batching

prioritization

Performance

memory pooling

tensor reuse

context caching

GPU pinning

These are allowed because they are compute optimizations, not authority.

10. Security Controls
Isolation

separate process

restricted OS user

sandboxed execution

seccomp/AppContainer where available

Authentication

All IPC calls require:

handshake token

runtime session id

Reject unknown callers.

Sanitization

Inputs must be:

size limited

schema validated

free of executable payloads

The runtime never executes arbitrary code from prompts.

11. Failure Behavior

The runtime must fail safely.

If failure occurs:

crash ? no data loss

restartable

stateless

no persistent corruption

Never:

hang indefinitely

hold locks

write partial data

silently retry dangerous actions

12. Observability

Allowed:

performance metrics

memory usage

inference time

error codes

Forbidden:

prompt logs

raw token logs

user content storage

Why:
Content logs violate sovereignty.

13. Model Policy

Models are treated as:

untrusted data blobs

Because:

weights are external artifacts

may contain unknown behavior

not authored by Hearthlink

Therefore:

models cannot gain privileges by existing.

They run inside the same sandbox.

Always.

14. API Contract

Minimal surface only.

Example:

POST /inference (IPC equivalent)

Request:

model_id

prompt_tokens

parameters

Response:

output_tokens

Nothing else.

No filesystem commands.
No tool calls.
No plugins.

Pure function.

15. Implementation Guidance (Rust)
Suggested structure

core-runtime/
src/
main.rs
ipc/
scheduler/
engine/
models/
memory/
Cargo.toml

No modules named:

auth

vault

synapse

plugins

network

If you see those, scope creep happened.

Recommended crates

tch / candle / llama.cpp bindings

serde

tokio

ipc channel lib

Avoid:

reqwest

hyper

filesystem traversal libs

anything network related

16. Relationship to Hearthlink Pillars
Data Sovereignty

No persistent storage or network ? cannot leak data

System Integrity

Sandbox + isolation ? cannot bypass boundaries

AI Governance

No authority ? cannot act independently

System Reliability

Stateless + restartable ? safe recovery

The runtime supports all pillars by being intentionally dumb.

17. Non-Goals

The runtime is NOT:

an agent

a plugin host

a connector layer

a policy engine

a memory store

a orchestrator

If it becomes any of these, it has failed its design.

18. One-Sentence Definition

The Hearthlink CORE Runtime is a sandboxed, offline inference engine that performs model execution only and has no authority over data, tools, or system actions.
