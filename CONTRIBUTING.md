# Contributing to qlevel

Thank you for contributing to `qlevel`, the open-source Commercial and Technical Bid Leveling Workbench for Quantity Surveyors and Procurement Engineers.

## Core Engineering Rules

1. **Zero Floating-Point Financial Math**: Every monetary rate, quantity, VAT rate, discount percentage, and leveled total in `qlevel-domain`, `qlevel-engine`, and `qlevel-workbook` must use fixed-point `rust_decimal::Decimal` (`DecimalValue`). `f32` and `f64` are strictly prohibited in commercial calculations.
2. **Zero Panics in Production Code**: `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, and `dbg!()` are denied via workspace Clippy lints. Return structured `Result<T, E>` errors across every crate boundary.
3. **Direct Cloud Multimodal Providers Only**: `qlevel` connects exclusively to OpenAI (`Sign in with ChatGPT` OAuth PKCE or API Key), Google Gemini (API Key), and Anthropic Claude (API Key). Do not add OpenRouter, proxy aggregators, or local unverified LLM runners.
4. **Dynamic Live Model Discovery**: Never hardcode static model dropdown lists in the frontend. Models are fetched live from each provider's `/models` endpoint and filtered in `qlevel-ai` to retain flagship Multimodal (text + vision/PDF) models only.
5. **Bilingual RTL and LTR Parity**: Every UI surface in `apps/desktop` and `packages/design-system` must use CSS logical properties (`ps-*`, `pe-*`, `ms-*`, `me-*`, `text-start`, `tabular-nums`) and full Arabic (`ar`) + English (`en`) translations.

## Local User-Space Setup (Windows 11, macOS, Linux)

No administrator privileges are required:

```bash
# 1. Install JavaScript workspace dependencies
pnpm install

# 2. Verify TypeScript bindings match Rust DTOs
pnpm bindings:check

# 3. Run Rust workspace tests and Clippy lints
pnpm rust:lint
pnpm rust:test

# 4. Typecheck frontend packages and launch desktop app
pnpm typecheck
pnpm dev
```

## Commit Message Convention

We enforce [Conventional Commits](https://www.conventionalcommits.org/) so `release-plz` and `git-cliff` calculate Semantic Versioning bumps automatically:

- `feat(engine): ...` - Minor version bump (`0.x.0` after `1.0.0`, patch/minor per config)
- `fix(parser): ...` - Bug fix
- `docs(specs): ...` - Documentation update
- `test(workbook): ...` - Test coverage addition
