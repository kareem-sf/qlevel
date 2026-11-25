# ADR-002: Zero-Trust Fixed-Point Decimal Arithmetic Across Rust and IPC

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

IEEE-754 binary floating-point (`f64` in Rust and `number` in JavaScript) introduces representation drift on base-10 currency fractions (for example `0.1 + 0.2 != 0.3`) and causes penny/halala/fils discrepancies across multi-million tender summaries. In addition, `ts-rs` does not ship a built-in `rust_decimal` feature ([ts-rs Issue #272](https://github.com/Aleph-Alpha/ts-rs/issues/272)), and LLMs frequently hallucinate arithmetic totals when asked to multiply BOQ tables.

## Decision

1. Wrap `rust_decimal::Decimal` in a canonical `DecimalValue(pub Decimal)` newtype in `qlevel-domain` annotated with `#[ts(type = "string")]` so values cross the Tauri IPC boundary as exact decimal strings.
2. Enforce `#![deny(clippy::float_arithmetic)]` on `qlevel-domain` and `qlevel-engine`.
3. Restrict AI models strictly to semantic extraction (matching row codes, classifying units, and quoting exclusion clauses). All multiplications, VAT stripping, Provisional Sum pinning, and missing-scope plugs run in deterministic Rust `Decimal` code, and exported Excel cells contain live formulas.

## Consequences

- Zero binary floating-point rounding drift across 2-decimal (`SAR`, `EGP`, `AED`, `QAR`, `USD`, `EUR`) and 3-decimal (`KWD`, `BHD`, `OMR`) currencies.
- Full auditability from extracted unit rate to Excel `SUM` total.
