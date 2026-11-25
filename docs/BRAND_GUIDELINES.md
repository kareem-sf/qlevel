# qlevel Brand Identity & Agentic Design System

## 1. Brand Essence & Logomark
- **Wordmark**: `qlevel` (strictly lowercase geometric typography).
- **Symbol (The Equilibrium Datum)**: Three stepped vertical bars in pure Alabaster (`#FAFAFA`) brought into exact horizontal parity along a razor-sharp datum line anchored by a centered Equilibrium Emerald node (`#10B981`), enclosed inside an Obsidian squircle (`#09090B`).
- **Voice**: Calm, authoritative procurement clarity. Zero internal engineering or cryptographic jargon in user-facing labels.

## 2. Color System (Geist Monochrome + Surgical Accents)
| Token | Light Mode | Dark Mode | Role |
| :--- | :--- | :--- | :--- |
| `--background` | `#FAFAFA` (Alabaster) | `#09090B` (Obsidian) | Primary application canvas |
| `--card` | `#FFFFFF` (Pure Surface) | `#121215` (Carbon Surface) | Floating surfaces, dialogs, A2UI cards |
| `--border` | `#E4E4E7` (Zinc 200) | `#27272A` (Zinc 800) | 1px hairline structural dividers |
| `--primary` | `#09090B` (Obsidian Ink) | `#FAFAFA` (Alabaster Ink) | Primary CTAs and active pills |
| `--emerald` | `#10B981` | `#34D399` | Live agent pulse and True Lowest Bidder indicator |
| `--amber` | `#F59E0B` | `#FBBF24` | Scope plugs, merged lump-sum prompts, hidden rows |
| `--destructive` | `#EF4444` | `#F87171` | Non-AVL brands, commercial exclusions, math traps |

## 3. Typography
- **Latin UI**: `Geist Variable` (`@fontsource-variable/geist`)
- **Arabic UI**: `IBM Plex Sans Arabic` (`@fontsource/ibm-plex-sans-arabic`)
- **Financial Figures & Excel Formulas**: `Geist Mono Variable` (`@fontsource-variable/geist-mono`) with `font-variant-numeric: tabular-nums` enforced on all rates, quantities, plugs, and totals.

## 4. UI Architecture Stack
- **Component Primitives**: `shadcn/ui` built on Radix UI (`@radix-ui/react-dialog`, `tabs`, `select`, `switch`, `tooltip`, `scroll-area`, `collapsible`) + `class-variance-authority` + `cmdk` + `sonner`.
- **AI-Native Patterns**: Vercel `ai-elements` (`https://github.com/vercel/ai-elements`) patterns (`PromptInput`, `Attachments`, `ChainOfThought`, `Checkpoint`, `Artifact`, `Sources`).
- **Declarative Agent-to-UI Protocol**: Google `A2UI` (`@a2ui/react` + `@a2ui/web_core` v0.16.1, `https://github.com/a2ui-project/a2ui`) driving the inline Merged Lump-Sum Decision Surface and the right-hand Source Evidence Inspector Surface via schema-validated JSON messages (`createSurface`, `updateDataModel`, `updateComponents`).
- **Data Grid & Split Layout**: `@tanstack/react-table` + `react-resizable-panels`.
