# Home screen dashboard — design

**Date:** 2026-10-12
**Author:** ALI88708 (contributor)
**Status:** awaiting review
**Scope:** `crates/ui-egui` only (L6). No engine, no PSD, no format crates.

---

## 1. Summary

PhotoCraft's Home screen is a fixed 460 × 330 px centred card with a New/Open pair, a drop hint and up to
six recent files. Adobe Photoshop's Home is a full-window dashboard: a template gallery, a recent-files
column with thumbnails, and a Learn panel. This spec replaces the fixed card with a responsive dashboard
that adds a **Templates** gallery driven by the presets that already exist in `new_doc_ui::CATEGORIES`,
while keeping the current card untouched as the narrow-window fallback.

The work is four independently reviewable pull requests. The first is a pure code move and changes no
behaviour, so the risky work only lands on top of a base that is already proven.

## 2. Why this, and why now

`docs/target-app-parity.md` records **discoverability and UI clarity as a ×0.92 multiplier** on the
essentials-user readiness number (78% × 0.90 × 0.92 × 0.95 ≈ 61%). The evidence it cites — *"language
option not found #2532, flyouts only on right-click #2301, toolbar hover issue #2603"* — is all
discoverability. The Home screen is the first thing a new user sees and the project currently ships none
of the affordances Photoshop has there.

`docs/scorecard.md` (UI-217-10, **partial**) already tracks the Home screen and names two shortfalls:
no canvas-pattern toggle (#2264) and no way to hide the Home screen recents. This spec closes the second
one by construction, because recents move into a column that can be laid out independently.

No onboarding, welcome, tutorial or empty-state work is tracked anywhere in the project today. This spec
adds the surface those would attach to, without inventing a first-run flow — see §3.

## 3. Goals and non-goals

**Goals**

1. Show **all 33 existing presets** (`new_doc_ui::CATEGORIES`, 7 categories) as a template gallery on the
   Home screen.
2. Make the Home screen responsive: dashboard at ≥ 900 px, today's card below that, with no layout
   clipping at the 760 × 480 minimum window size (`apps/photocraft/src/main.rs:76`).
3. Move Home-screen code out of `canvas.rs` (6,000+ lines) into a module with one job.
4. Preserve today's card behaviour byte-for-byte so the fallback is provably the current UI.
5. Give agents a readable surface: the selected category lives in serde state, so the control channel
   and MCP can read it and drive it.

**Non-goals**

- **No first-run wizard, tutorial, or onboarding flow.** That is a separate design problem with its own
  open questions; the project tracks nothing for it and this spec must not smuggle one in.
- **No bundled template images.** See §5.
- **No real thumbnails for recent files.** Deferred to PR 4 (§9) because it needs a decode-and-cache
  path, not UI code.
- **No changes to the engine, command registry, `canvas.rs` logic, or any shared state schema beyond one
  field.** Every new user-visible behaviour here opens an *existing* dialog or dispatches an *existing*
  command; nothing new is registered.
- **No dark/light theme work, no new `Tokens`.** All colours and radii come from `theme::Tokens`
  (AGENTS.md golden rule 5).

## 4. What exists today

| Thing | Where |
|---|---|
| Home screen root | `crates/ui-egui/src/canvas.rs:2369` `start_screen` |
| Recent-files list | `crates/ui-egui/src/canvas.rs:2443` `home_recent` |
| Drop hint (Wayland branch) | `crates/ui-egui/src/canvas.rs:2427` `start_screen_drop_hint` |
| Home visibility rule | `crates/ui-egui/src/chrome_ui.rs:29` `ChromeState::shows_home` |
| Home button (Pro theme) | `crates/ui-egui/src/chrome_ui.rs:199` |
| Preset categories (33 presets) | `crates/ui-egui/src/new_doc_ui.rs:19` `CATEGORIES` |
| Apply a preset to dialog fields | `crates/ui-egui/src/new_doc_ui.rs:113` `apply_preset` |
| Preference that drives Home | `crates/engine/src/prefs.rs:239` `interface.auto_show_home_screen` (default `true`) |

Sizing today is hard-coded: card `460 × (330 + recent_h)` (`canvas.rs:2376`), buttons `190 px`
(`canvas.rs:2395`, `:2397`, `:2401`), recent rows `380 px` wide and `30 px` tall (`canvas.rs:2436-2438`,
`:2445`).

## 5. Design

### 5.1 Responsive layout

The canvas area decides the layout from `ui.available_rect_before_wrap()`.

| Available width | Layout |
|---|---|
| `< 900 px` | `compact_card` — today's 460 px card, **unchanged** |
| `≥ 900 px` | `full_dashboard` — three columns |

900 px is chosen so that at the 760 px minimum window the fallback is guaranteed, and at the 1440 × 900
default window the dashboard has room for the three columns defined below.

### 5.2 `full_dashboard` columns

```
┌──────────┬────────────────────────────────┬──────────────┐
│ New      │  Templates                     │  Recent      │
│ Open     │  [chips: 7 categories]         │  (rows)      │
│          │  ┌────┐┌────┐┌────┐            │              │
│ Recent   │  │card││card││card│            │  Learn       │
│ (rows)   │  └────┘└────┘└────┘            │  What's New  │
└──────────┴────────────────────────────────┴──────────────┘
   200 px              flexible                    260 px
```

Column widths are fixed by `Tokens` spacing conventions; the templates column takes the remainder.
The columns collapse into a single scrollable column rather than clipping if the height is short —
this is what prevents clipping at 760 × 480 in the wide layout at minimum window height.

### 5.3 Templates panel

- One **active category** at a time, selected by a chip row across the top. `Recent` is active on open.
- Below the chips, a grid of cards for that category.
- Each card shows a **shape drawn at the preset's real aspect ratio**, with the preset name and its
  pixel dimensions under it. No images.
- Clicking a card opens the **existing** `DialogKind::NewDocument` dialog with `apply_preset` already
  applied, so the user can adjust the size before committing. This matches Photoshop, where a template
  click opens a dialog rather than creating silently.
- `Browse all templates →` is not shipped in the first dashboard PR: with one active category the grid
  is already reachable, and a second entry point to the same data would be redundant.

**Why no images.** Template artwork would be bundled assets: a licence file beside each one and a row
in `ATTRIBUTION.md` (AGENTS.md golden rule 3). Shape plus dimensions delivers most of the value at zero
assets and zero licences, and the card stays crisp at every DPI because egui paints it.

**Why one category at a time.** Showing all 33 in a single grid makes the Home screen a scroll of 33
near-identical rectangles; the user's own screens are usually a handful of sizes. Tabs make the common
case reachable without scrolling and leave room for the template list to grow later.

### 5.4 What moves, and what changes

`home_screen.rs` (new) receives `start_screen`, `home_recent` and `start_screen_drop_hint` **verbatim**
from `canvas.rs`. `canvas.rs` loses ~145 lines and calls `home_screen::screen(app, ui)` instead. The only
behavioural addition is `full_dashboard`, reached only at ≥ 900 px.

## 6. Components

```
crates/ui-egui/src/home_screen.rs
├── screen(app, ui)        picks compact vs full from the available width
├── full_dashboard(ui)     the layout above
├── compact_card(ui)       today's card, moved unchanged
├── templates(ui)          chips + preset grid
├── recents(ui)            today's home_recent, moved unchanged
└── learn(ui)              links + version line   (PR 3)

crates/ui-egui/src/home_screen_tests.rs   (new)
```

One field of new state, appended to the struct that already holds the Home screen's state:

```rust
// crates/ui-egui/src/chrome_ui.rs:14
pub struct ChromeState {
    pub status_info: String,
    pub home: Option<usize>,
    pub home_category: usize,   // index into new_doc_ui::CATEGORIES; 0 = "Recent"
}
```

It lives in `state.rs`, which is serde, so the control channel and MCP server can read and set it
(AGENTS.md golden rule 5) — the same affordance the project already relies on for its own screenshots.

## 7. Data flow

```
template card clicked
  → apply_preset(name, w, h, ppi, fields, units)        new_doc_ui.rs:113  (existing)
  → app.open_dialog(DialogKind::NewDocument, fields)   (existing)
  → user adjusts, presses Create
  → file.new                                          (existing command)
```

Three calls, all existing. No new command, no new dialog, no engine change.

The recent-files path is untouched: `recents` calls `app.open_path`, and a failure goes to
`app.open_failed(name, &e)` exactly as `home_recent` does today (`canvas.rs:2508-2512`).

## 8. Error handling

The never-crash rule (AGENTS.md golden rules, "Never crash") outranks everything here. Specifically:

| Condition | Handling |
|---|---|
| `home_category` out of range (e.g. stale state file, or a set value from the control channel) | `CATEGORIES.get(i)` with `.unwrap_or(&CATEGORIES[0])` — never `[]` |
| Window narrower than 900 px | compact card; the dashboard is never laid out |
| Window shorter than the content needs | single scroll column; never clipped |
| A preset with extreme dimensions (A3 = 3508 × 4961) | `file.new` already validates 1–300,000 px; the dialog owns the failure |
| Rapid repeated clicks | each click builds a fresh dialog from a fresh field map; no accumulated state |
| `cfg(target_arch = "wasm32")` | templates are pure constants and render normally; only recents are empty, unchanged from today |
| New `chrome_category` value from the web build or a corrupt prefs file | `serde` default of `0`; out-of-range handled by the `.get()` above |

No `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!` or `unimplemented!` outside tests
(`clippy.toml` enforces this crate-wide).

## 9. PR sequence

| PR | Contents | Size | Risk |
|---|---|---|---|
| **1** | Move `start_screen`/`home_recent`/`start_screen_drop_hint` into `home_screen.rs`; add the width threshold and route to the moved card. **No visible change.** | small | none — pure move |
| **2** | `full_dashboard` + `templates`. Adds `home_category`. | medium | low |
| **3** | `learn` column (documentation links and a What's New line). | small | low |
| **4** | Recent-file thumbnails — deferred, needs a background decode and cache in `L4`, not `L6`. | large | medium |

PR 1 lands first deliberately: a behaviour-neutral move. If it is rejected or breaks something, that is
discovered immediately, before any visual change exists to unpick.

## 10. Testing

`home_screen_tests.rs`, using the same `kittest` harness the crate already uses
(`crates/ui-egui/src/direction_tests.rs:97`):

| Test | Asserts |
|---|---|
| `all_templates_are_reachable` | every preset in `CATEGORIES` renders a card; total equals the category sum |
| `narrow_window_uses_the_compact_card` | at 760 px wide the compact card draws and the dashboard does not |
| `wide_window_uses_the_dashboard` | at 1440 px wide the dashboard draws |
| `switching_category_keeps_the_card_count` | selecting each of the 7 categories yields exactly that category's preset count |
| `nothing_clips_at_the_minimum_window` | at 760 × 480 every label rect lies inside the available rect |
| `an_out_of_range_category_falls_back_to_recent` | `home_category = 999` renders the Recent cards and does not panic |

Existing tests that must stay green: `chrome_ui.rs:380-420` (Home shows/closes, auto-show off) and
`canvas.rs:6000-6013` (Wayland hint, removing a recent entry).

A new string introduced by this work (the chip labels reuse the existing `CATEGORIES` names, so there
should be none) must be added to all 17 catalogs or the `i18n` gates fail
(`i18n/mod.rs:751`, `:883`, `:937`). The template chip labels reuse the category names that the New
Document dialog already translates, so PR 2 should add **zero** new strings.

## 11. Verification gate

Per AGENTS.md §5, for the crates touched:

```sh
cargo test -p photocraft-ui-egui --lib
cargo clippy -p photocraft-ui-egui --all-targets -- -D warnings
cargo xtask layers
cargo xtask wasm                    # L6 must still compile to wasm32
cargo run -p photocraft-ui-egui --example snapshot   # look at the PNG
```

Plus a visual pass through the running app at 1440 × 900, 900 × 600 and 760 × 480, and a pass in the dark
Pro theme (the Home button only appears there, `chrome_ui.rs:199`).

No new command is registered, so `cargo xtask parity`, `panic_hunt` and `cargo xtask scorecard` are not
required by this change. Docs that move a number are updated in the same PR: `docs/ui-parity.md` and
`docs/scorecard.md` UI-217-10.

## 12. Risks and open questions

| Risk | Mitigation |
|---|---|
| Templates are blank sizes, not artwork — a user may expect designed layouts. | The card labels the dimensions; Photoshop's own New dialog is sizes too. If designed templates are wanted later that is a separate, larger feature. |
| The dashboard is a new layout with no Photoshop 1:1 reference for the *Home screen itself*. | Kept to the columns Photoshop uses; any divergence is visible in review. |
| `Learn` content has no source in-repo. | PR 3 links the published docs only; no new assets. |
| Thumbnail work (PR 4) crosses into `L4` decode paths. | Explicitly out of scope here; gets its own spec. |

**Open question for the maintainer:** `docs/scorecard.md` UI-217-10 also asks for a canvas-pattern
toggle (#2264), which lives in Preferences › Interface, not on the Home screen. This spec covers the
"hide the Home recents" half and leaves the pattern toggle alone.