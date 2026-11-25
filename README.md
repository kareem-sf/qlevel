# qlevel - Agentic Commercial & Technical Bid Leveling Workbench

**Open-source desktop workbench for Quantity Surveyors, Cost Managers, and Procurement Engineers to equalize subcontractor quotations (`CBE`) and catch hidden technical exclusions (`TBE`) into a live-formula Excel comparison sheet.**

- **License**: Apache-2.0 (`LICENSE`, `NOTICE`, `TRADEMARKS.md`)
- **Core Engine**: Tauri v2 + Pure Rust (`rust_decimal` fixed-point math, `calamine`, `lopdf`, `rust_xlsxwriter`, `rusqlite` WAL + `BLAKE3`)
- **Agentic UI Stack**: React 19 + TypeScript + Tailwind CSS v4 + **shadcn/ui** (Radix UI) + **Vercel AI Elements** (`vercel/ai-elements`) + **Google A2UI** (`@a2ui/react` & `@a2ui/web_core` v0.16.1) + `@tanstack/react-table` v9 + `react-resizable-panels` + `cmdk` + `sonner` + `Geist` & `IBM Plex Sans Arabic`
- **Platforms**: Linux (`Arch Linux` / `Ubuntu 22.04+` portable `.AppImage`, `.deb`, `.rpm`), Windows 10/11 (`.exe` portable + `.msi` / NSIS), macOS 13+ (Apple Silicon & Intel `.dmg`)
- **Zero Server Custody**: Runs 100% locally on your workstation using your own direct OpenAI (`Sign in with ChatGPT` OAuth PKCE or API Key), Google Gemini, or Anthropic Claude account.

## Minimalist Agentic UI Architecture

1. **Agentic Command Intake (`Screen 1`)**: Calm monochrome `#FAFAFA` / `#09090B` composer built with Vercel `ai-elements` (`PromptInput`, `Attachments`, `AttachmentChip`), supporting drag-and-drop Excel/PDF tenders, `Ctrl+V` clipboard table/image paste, inline role cycling (`Master` / `Vendor` / `AVL`), and quick-config pills.
2. **Live Split-Pane Reasoning & Inline A2UI Decisions (`Screen 2`)**: Resizable split workspace pairing a live Vercel `ChainOfThought` Agent Stream and Google **A2UI** (`<A2uiSurface />`) inline Merged Lump-Sum Decision Surface on the left with a live Commercial Equalization `Artifact` grid on the right - zero blocking popup modals.
3. **Equalized Results & A2UI Source Evidence Inspector (`Screen 3`)**: `@tanstack/react-table` v9 comparison matrix, emerald True Lowest Bidder savings bar, live Excel formula bar preview, graceful `_v2.xlsx` file-lock fallback, and a right-hand Google **A2UI** Source Evidence Inspector highlighting verbatim PDF/Excel exclusion clauses.
4. **Floating Command Sheet & `Ctrl+K` Palette (`Screen 4`)**: Minimal `shadcn/ui` settings popover (`AI Providers`, `Tender Rules`, `Excel Output`) and global `cmdk` keyboard palette (`Ctrl+K` / `Cmd+K`).

## Running on Arch Linux, Windows & macOS

Download the latest release from [GitHub Releases](https://github.com/kareem-sf/qlevel/releases):

```bash
# Arch Linux / Any x86_64 Linux (Portable AppImage)
chmod +x qlevel_0.0.2_amd64.AppImage
./qlevel_0.0.2_amd64.AppImage
# Note: On Arch Linux without fuse2 installed, run either:
# sudo pacman -Syu fuse2   OR   ./qlevel_0.0.2_amd64.AppImage --appimage-extract-and-run
```

## Local Development

Prerequisites: [Rust 1.85+](https://rustup.rs/), [Node.js 20+](https://nodejs.org/), and [pnpm 9+](https://pnpm.io/).

```bash
git clone https://github.com/kareem-sf/qlevel.git
cd qlevel

pnpm install
pnpm -r typecheck
cargo test --workspace
pnpm dev
```

## Documentation Index

- [Brand Identity & Agentic Design System (`BRAND_GUIDELINES.md`)](docs/BRAND_GUIDELINES.md)
- [System Architecture & Data Flow](docs/architecture/OVERVIEW.md)
- [Commercial Leveling Math Specification (RICS NRM 2 & GCC/Egypt VAT)](docs/specs/COMMERCIAL_MATH_SPEC.md)
- [Technical Trap & Scope-Exclusion Taxonomy](docs/specs/TECHNICAL_TRAPS_SPEC.md)
- [Excel 2016+ Workbook Formula & Layout Specification](docs/specs/EXCEL_WORKBOOK_SPEC.md)
- [Arabic & English Engineer Quickstart Guide](docs/guides/QUICKSTART_AR_EN.md)
- [Architecture Decision Records (`ADR-001` to `ADR-006`)](docs/adr/)

## License & Attribution

Licensed under the [Apache License, Version 2.0](LICENSE). Redistribution in source or binary form must preserve [NOTICE](NOTICE) and comply with [TRADEMARKS.md](TRADEMARKS.md).
