# parson_port

Idiomatic Rust port of [kgabis/parson](https://github.com/kgabis/parson) — a lightweight C89 JSON parser/serializer (~2,500 LOC).

**Track A: C → Rust** — Port Mortem 2026 hackathon submission.

## Goals

- Zero `unsafe` blocks in core parsing logic
- 100% behavioural parity with the original C library
- Original `tests.c` preserved unmodified under `tests/original/`
- RFC 8259 conformance improvements over the C original (documented in DECISIONS.md)

## Build

```sh
cargo build
cargo test
```

## Original source

Vendored at kickoff SHA: `ba29f4eda9ea7703a9f6a9cf2b0532a2605723c3`
