# memd Dashboard — Design System

Blended from Superhuman (purple confidence), SpaceX (aerospace minimalism), and Warp (warm dark restraint).

## 1. Visual Theme & Atmosphere

The memd dashboard feels like a mission control terminal built by someone who reads Monocle — warm darkness, quiet confidence, and a single accent color used with surgical precision. The canvas is a warm near-black (not the cold void of pure #000, not the earth tones of a terminal — something in between, with a faint violet undertone that hints at the system's intelligence). Text glows in spectral warm-white with a barely-perceptible cream cast, never harsh pure white.

Purple is the singular accent — a restrained lavender that appears only where attention is earned: active states, live data pulses, the connection indicator. Everything else is monochromatic warm grays. The design communicates through typography hierarchy and spatial rhythm, not decoration.

**Key Characteristics:**
- Warm void background (`#0a0a0f`) with violet undertone — not cold black, not warm brown
- Spectral parchment text (`#f0f0f5`) — near-white with faint violet warmth, never pure `#fff`
- Violet accent (`#8b5cf6`) used with extreme restraint — active states, badges, the single glow
- Zero shadows — depth from semi-transparent borders and surface opacity shifts only
- Uppercase micro-labels with wide tracking (1.2px+) for system taxonomy — aerospace stencil voice
- Tight display compression (line-height 0.95–1.0) contrasted with generous body spacing (1.5)
- Glass surfaces at 4% white overlay — barely perceptible layer separation
- Binary radius: 8px (controls) and 12px (panels) — no pills, no micro-rounding

## 2. Color Palette & Roles

### Primary
- **Void** (`#0a0a0f`): Page canvas — warm near-black with faint violet cast
- **Surface** (`#12121a`): Panel/card backgrounds — one step above void, violet-tinted
- **Parchment** (`#f0f0f5`): Primary text — spectral near-white, warm but not cream

### Accent (singular)
- **Violet** (`#8b5cf6`): The only accent color. Active nav, live indicators, badge highlights
- **Violet Bright** (`#a78bfa`): Hover/emphasis variant — lighter, more luminous
- **Violet Glow** (`#8b5cf640`): Ambient glow behind active elements — 25% opacity

### Surfaces & Borders
- **Glass** (`rgba(255, 255, 255, 0.04)`): Panel overlay — barely-there surface differentiation
- **Border Subtle** (`rgba(139, 92, 246, 0.15)`): Default card/panel borders — violet-tinted at 15%
- **Border Active** (`rgba(139, 92, 246, 0.50)`): Active/focused element borders — violet at 50%

### Text Hierarchy
- **Text Primary** (`#f0f0f5`): Headlines, high-emphasis content
- **Text Secondary** (`#8888a0`): Body text, descriptions — warm mid-gray with violet lean
- **Text Tertiary** (`#555570`): Metadata, timestamps, muted labels — recedes but legible

### Semantic Status
- **Current/Active** (`#34d399`): Healthy, connected, live — emerald
- **Stale/Warning** (`#fbbf24`): Needs attention, aging — amber
- **Expired/Error** (`#f87171`): Failed, disconnected, expired — coral red
- **Candidate/Info** (`#60a5fa`): Pending review, new — soft blue

### Reserved
- No gradients on dashboard surfaces — flat color with border containment
- No additional accent colors — violet is the singular brand gesture
- Status colors appear ONLY on status indicators, never on backgrounds or large surfaces

## 3. Typography Rules

### Font Families
- **Display & Body**: `Inter` — geometric, clean, widely available. The workhorse.
- **Mono**: `JetBrains Mono` — for IDs, code, system values, timestamps

### Hierarchy

| Role | Size | Weight | Line Height | Letter Spacing | Notes |
|------|------|--------|-------------|----------------|-------|
| Page Title | 24px | 600 | 1.0 | -0.5px | Compressed, confident |
| Section Head | 14px | 500 | 1.2 | 0px | Panel headers, group labels |
| Body | 14px | 400 | 1.5 | 0px | Standard content — generous spacing |
| Body Small | 13px | 400 | 1.4 | 0px | Descriptions, secondary content |
| Label | 11px | 500 | 1.0 | 0px | Badge text, kind/stage tags |
| Micro Label | 11px | 400 | 1.2 | 0.8px | Uppercase metadata (scope, timestamps) |
| Mono ID | 12px | 400 | 1.0 | 0px | UUIDs, hashes — JetBrains Mono |
| Mono Value | 11px | 400 | 1.0 | 0px | Small system values — JetBrains Mono |

### Principles
- **Regular weight dominance**: Weight 400 for nearly everything. 500 for emphasis. 600 only for page titles. Never 700 — bold has no place in this system.
- **Uppercase as taxonomy**: Scope labels, kind categories, and system metadata use uppercase + wide tracking. This is the aerospace stencil voice from SpaceX — every label feels stamped.
- **Tight display, generous body**: Page titles compress at 1.0 line-height (Superhuman's architectural density). Body text breathes at 1.5 (Superhuman's reading comfort). The contrast is the system.
- **Negative tracking on display only**: -0.5px on 24px titles. Body text stays at 0. Larger text, tighter tracking.
- **Mono for system truth**: Anything the machine generated (IDs, timestamps, hashes, confidence scores) renders in JetBrains Mono. Anything a human wrote renders in Inter.

## 4. Component Stylings

### Panels (Glass)
- Background: `Surface` (`#12121a`) at 40% opacity over `Void`
- Border: `Border Subtle` — 1px, violet-tinted at 15% opacity
- Radius: 12px
- Padding: 20px
- Hover (interactive panels): border shifts to `Border Active`, faint `Violet Glow` box-shadow
- No inner shadows, no blur effects, no glassmorphism — the name "glass" is metaphorical, not literal

### Metric Cards
- Same glass panel base
- Value: 30px, weight 600, tabular-nums — the number dominates
- Label: 14px, weight 400, `Text Secondary`
- Sub-label: 12px, `Text Tertiary`
- Accent color on value text for semantic meaning (emerald for healthy, violet for count)

### Badges (Kind, Stage, Status)
- Background: accent color at 15% opacity
- Text: accent color at full or 80% brightness
- Border: accent color at 30% opacity
- Radius: 4px (tight, not pill)
- Font: 11px, weight 500
- Each memory kind gets a unique hue from the extended palette — but all rendered at 15% bg / 30% border to maintain restraint

### Buttons
- **Primary (Dark Fill)**: `#353534` background (Warp's Earth Gray), `Text Secondary` text, 8px radius, 500 weight
- **Ghost**: transparent background, `Border Subtle` border, `Text Secondary` text, 8px radius — SpaceX's ghost button DNA
- **Danger**: `Expired` color at 15% background, `Expired` text, 30% border — same badge pattern
- **Success**: `Current` color at 15% background, `Current` text, 30% border
- Hover: opacity shift only — no color transformations, no scale, no bounce
- Padding: 6px 12px for inline, 10px 16px for standalone

### Status Indicator (Dot)
- 6px circle, semantic color fill
- Adjacent text: 11px, `Text Secondary`
- No animation on the dot — steady, calm, not pulsing. The system is confident, not anxious.

### Inputs
- Background: `Void` (`#0a0a0f`)
- Border: `Border Subtle`
- Text: `Text Primary`
- Placeholder: `Text Tertiary`
- Focus: border to `Violet`, no glow ring — one clean state change
- Radius: 8px

### Navigation (Sidebar)
- Width: 208px fixed
- Background: `Surface` at 60% opacity
- Links: 14px, weight 400, `Text Secondary`
- Active link: `Violet Bright` text, `Violet` bg at 10%, `Border Active` left or border
- Brand mark: 18px, weight 600, `Violet` color — the only large violet text element
- Footer: 12px, `Text Tertiary`

### Tables / List Rows
- Row border: `Border Subtle` bottom, last child no border
- Row padding: 6px 0 vertical
- Hover: `rgba(255, 255, 255, 0.02)` background — barely visible
- No zebra striping — the borders provide sufficient separation

### Confidence Bar
- Track: `rgba(255, 255, 255, 0.05)` — 4px height, full radius
- Fill: emerald (≥80%), amber (50–79%), coral (<50%)
- Value label: 11px mono, `Text Tertiary`

## 5. Layout Principles

### Spacing System
- Base: 4px
- Scale: 2, 4, 6, 8, 12, 16, 20, 24, 32, 48px
- Section padding: 32px between major sections
- Panel padding: 20px internal
- Component gaps: 8–12px between related elements
- Page padding: 32px from edges

### Grid
- Max content width: 1152px (72rem)
- Metric cards: 4-column grid, 16px gap
- Single-column content with sidebar — no complex multi-column layouts
- Full-width panels within the content column

### Whitespace Philosophy
- **Warp's editorial calm**: Generous spacing between sections. The dark void is not empty — it's restful.
- **Density where earned**: Working memory records, search results, and inbox items are dense (tight row spacing). Overview metrics and section headers breathe.
- **Progressive disclosure**: Summary → click to expand detail. Never dump all data at once.

### Border Radius Scale
- 4px: Badges, tags, small inline elements
- 8px: Buttons, inputs, inline controls
- 12px: Panels, cards, major containers
- No pills (50px+). No micro-rounding (2px). No sharp corners (0px).

## 6. Depth & Elevation

| Level | Treatment | Use |
|-------|-----------|-----|
| L0 Void | Flat `#0a0a0f` | Page canvas |
| L1 Surface | `#12121a` at 40% + 1px `Border Subtle` | Panels, cards, sidebar |
| L2 Hover | `rgba(255,255,255,0.02)` | Row hover, interactive feedback |
| L3 Active | `Border Active` + `Violet Glow` shadow | Focused/selected elements |

### Shadow Philosophy
Zero box-shadows in the default state. Following SpaceX's doctrine: in a dark interface, shadows are invisible and meaningless. Depth is communicated through:
- **Border opacity tiers**: 15% (resting) → 50% (active) — the border IS the shadow
- **Surface opacity tiers**: 0% (void) → 4% (glass) → 40% (surface) — stacked transparency
- **Color as depth**: Violet elements read as "forward" against the gray palette

Exception: Active/selected panels may use `0 0 20px rgba(139, 92, 246, 0.08)` — a faint violet glow. This is the only shadow in the system. It's earned, not ambient.

## 7. Do's and Don'ts

### Do
- Use `#f0f0f5` for text, never pure `#ffffff` — the warmth is subtle but essential
- Keep violet (`#8b5cf6`) as the singular accent — no second accent color
- Use uppercase + tracking for taxonomy labels only (scope, kind, timestamps) — body text is sentence case
- Apply 15% opacity backgrounds on semantic badges — restraint even in color
- Let borders do the work of shadows — opacity shift, not blur
- Use tabular-nums on all numeric values — alignment is information
- Use mono font for machine-generated values, sans-serif for human content
- Keep weight ≤600 — no bold, no heavy. Confidence is quiet.

### Don't
- Use pure black (`#000000`) for backgrounds — `#0a0a0f` has violet warmth
- Add gradients, glows, or blur effects to surfaces — depth is flat + border
- Use bold (700+) weight anywhere — 600 is the ceiling, 400 is the floor
- Apply pill radius (50px+) to any element — max is 12px
- Make status dots pulse or animate — the system is calm, not anxious
- Add hover scale transforms — opacity shift only
- Introduce color outside the defined palette — new features use the existing system
- Use icon-heavy UI — text labels and badges, not icon grids

## 8. Responsive Behavior

### Breakpoints
| Name | Width | Changes |
|------|-------|---------|
| Compact | <768px | Sidebar collapses to top bar, single-column, reduced padding |
| Standard | 768–1280px | Full sidebar, content column, 4-col metrics |
| Wide | >1280px | Max-width container centered, generous side margins |

### Touch Targets
- Buttons: min 36px height
- Nav links: 36px row height with full-width click area
- Badge buttons: adequate padding despite small text
- Checkboxes: 16px with 4px surrounding padding

### Collapsing Strategy
- Sidebar → horizontal top nav on compact
- 4-column metric grid → 2-column → stacked
- Expanded detail views → full-width on compact
- Panel padding reduces 20px → 16px → 12px

## 9. Agent Prompt Guide

### Quick Color Reference
- Background: `#0a0a0f` (void), `#12121a` (surface)
- Text: `#f0f0f5` (primary), `#8888a0` (secondary), `#555570` (tertiary)
- Accent: `#8b5cf6` (violet), `#a78bfa` (bright), `#8b5cf640` (glow)
- Border: `rgba(139,92,246,0.15)` (subtle), `rgba(139,92,246,0.50)` (active)
- Status: `#34d399` (ok), `#fbbf24` (warn), `#f87171` (error), `#60a5fa` (info)

### Design DNA
- **From Superhuman**: Purple as singular accent. Tight display compression. Confidence through emptiness. Binary radius system.
- **From SpaceX**: Zero shadows. Uppercase taxonomy labels. Ghost/translucent interactives. Spectral text color. Mission-critical restraint.
- **From Warp**: Warm dark canvas. Semi-transparent borders over shadows. Regular weight dominance. Editorial spacing. Monochromatic calm.

### What memd adds
- **System taxonomy as color**: Each memory kind gets a unique hue, but expressed at 15% opacity — color as classification, not decoration
- **Machine vs human typography**: Mono for system values, sans for human content — the typeface tells you who wrote it
- **Calm confidence**: No loading spinners, no skeleton screens, no animations. Data appears or it doesn't. The system trusts itself.
