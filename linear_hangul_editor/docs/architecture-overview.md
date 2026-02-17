# Linhan Architecture Overview

## Snapshot
- App type: Tauri v2 desktop app with SvelteKit frontend and Rust backend.
- Primary purpose: edit Hangul glyph/config/kerning data, compile a generated font, and preview text with that font.
- Frontend entrypoint: `src/routes/+page.svelte`.
- Backend entrypoint: `src-tauri/src/main.rs` -> `linhan_lib::run()`.

## High-Level Shape
- `src/`: Svelte UI and editors.
- `src-tauri/`: Rust commands, font-generation logic, filesystem helpers, app config/capabilities.
- Seed data in repo root:
  - `configs/` (JSON5)
  - `kernings/` (txt)
  - `glyph_sets/` (Lua files per glyph)
  - `contents/` (sample text)
  - `toolsets/` (preset mappings)
  - `fonts/` (font artifacts)

## Frontend Structure

### Main screen
`src/routes/+page.svelte` is a single orchestrator page that:
- Loads toolsets, glyph sets, content names, and available fonts at mount.
- Registers Tauri event listeners:
  - `msg` for `compile_ended`
  - `error` for compile/runtime errors
- Invokes backend commands for CRUD on toolsets/configs/kernings/glyph sets/content/fonts.
- Triggers compile via `invoke("run_compile", { glyphSet, configName, kerningName })`.
- Previews content in two textareas (default + custom font "Linear Korean").

### Child editors
- `src/lib/config_editor.svelte`
  - `get_config_names`, `get_config_data`, `save_config`
- `src/lib/kerning_editor.svelte`
  - `get_kerning_names`, `get_kerning_data`, `save_kerning_data`
- `src/lib/glyph_data_editor.svelte`
  - `get_glyph_data`, `save_glyph_data`
  - works with one selected glyph at a time from a fixed dropdown list

## Tauri Command Surface
All commands are registered in `src-tauri/src/lib.rs` (`tauri::generate_handler!`).

Command groups:
- Compile: `run_compile`
- Toolsets: `get_tool_set_names`, `get_tool_set_data`, `save_tool_set`, `delete_tool_set`
- Glyph sets: `get_glyph_set_names`, `get_glyph_data`, `save_glyph_data`, `copy_glyph_set`, `delete_glyph_set`
- Configs: `get_config_names`, `get_config_data`, `save_config`
- Kernings: `get_kerning_names`, `get_kerning_data`, `save_kerning_data`
- Contents: `get_content_names`, `get_content`, `save_content`
- Fonts: `get_font_names`, `get_font_data`, `save_font`, `delete_font`

## Compile Pipeline (Rust)

### Runtime sequence
1. UI saves editor states (kerning/config/glyph editors).
2. UI calls `run_compile`.
3. `run_compile`:
   - loads/parses selected config JSON5,
   - builds `Args` (`get_args`) including kerning map,
   - spawns a background thread.
4. Thread deletes previous generated font directory (`generated`) if present.
5. Calls `compile(args, glyph_set)`:
   - creates/loads font tables,
   - builds base glyphs from Lua definitions,
   - generates compatibility jamos,
   - generates Hangul syllable composites (U+AC00..U+D7A3),
   - updates font tables (`head/hhea/maxp/name/post`),
   - writes `generated.ttf`.
6. Converts TTF -> WOFF2 in-process via Rust crate `ttf2woff2`.
7. Copies compile inputs into generated font dir:
   - selected config -> `config.json5`
   - selected kerning -> `kerning.txt`
   - selected glyph set -> `glyph_data/`
8. Emits `msg: compile_ended` or `error` event.

### Core modules
- `src-tauri/src/lib.rs`: command handlers + compile orchestration.
- `src-tauri/src/file.rs`: path and filesystem utilities.
- `src-tauri/src/glyph.rs`: Lua glyph parsing + simple glyph creation.
- `src-tauri/src/compose.rs`: component placement and kerning-aware composition.
- `src-tauri/src/font.rs`: table loading/merging/generation/writing.
- `src-tauri/src/consts.rs`: constants + conversion/compatibility maps.
- `src-tauri/src/structs.rs`: `Args`, `Config`, `ToolSet`, `FontTables`.

## Data Model and Storage

### Runtime root
- Runtime data root is the platform app-data location returned by Tauri `app.path().app_data_dir()`.
- On first run (or if a subfolder is missing), app seeds from bundled Tauri resources.
- In debug/dev builds, if bundled resources cannot be resolved, seeding falls back to repo seed directories.

### Typical runtime layout
- `<app_data_dir>/toolsets/*.toolset`
- `<app_data_dir>/configs/*.json5`
- `<app_data_dir>/kernings/*.txt`
- `<app_data_dir>/glyph_sets/<set_name>/*.lua`
- `<app_data_dir>/contents/*`
- `<app_data_dir>/fonts/<font_name>/<font_name>.ttf|woff2`

### File format notes
- Toolset file: `config_name kerning_name glyph_set` relationship.
- Kerning txt line format: `<cho_char>,<jung_char>,<jong_char_as_cho>,<float_kern>`.
- First 3 columns may be empty and each column can contain multiple values split by `|`.
- Third column uses cho-form consonants and is converted to jong internally (e.g. `ㄱ` -> `ᆨ`).
- One kerning pair is produced per line: scanning left to right, first use two values from one column; otherwise use two non-empty adjacent columns. The first pair found is used.
- Glyph Lua files return `curves` arrays of points `[x, y, on_curve_flag]`.
- Config JSON5 controls dimensions/ratios/gaps/stroke behavior and optional source font.

## App Configuration / Build
- Frontend build/dev scripts: `package.json` (`dev`, `build`, `tauri`).
- Tauri wiring: `src-tauri/tauri.conf.json`.
- Seed directories are bundled via `bundle.resources` in `src-tauri/tauri.conf.json`.
- SvelteKit configured as static CSR (no SSR): `src/routes/+layout.js`.

## Security / Capabilities
- Capability file: `src-tauri/capabilities/default.json`.
- Includes broad FS permissions (`fs:read-all`, `fs:write-all`) and `$HOME` scope.

## Practical Orientation Checklist
When re-entering this codebase, start here:
1. `src/routes/+page.svelte` to understand user flows and invoked commands.
2. `src-tauri/src/lib.rs` to map each command and compile orchestration.
3. `src-tauri/src/glyph.rs` + `compose.rs` + `font.rs` for generation internals.
4. `src-tauri/src/file.rs` and `src-tauri/src/consts.rs` for storage paths and defaults.
5. Seed data under `configs/`, `kernings/`, `glyph_sets/`, `toolsets/`, `contents/`.

## Operational Dependencies
- WOFF2 conversion is performed in-process via Rust crate `ttf2woff2` (no external `fonttools` requirement).
