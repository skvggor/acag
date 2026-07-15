# AGENTS.md

Guidance for AI agents working in this repository.

## What this is

`acag` (article cover art generator) is a native desktop app that generates
article cover art in five aspect ratios (square, link/Open Graph, wide, portrait,
banner) in an Omakase / Japanese-constructivist style. Rust +
[Slint](https://slint.dev) for the GUI, [resvg](https://github.com/linebender/resvg)
/ tiny-skia for rasterization. Single binary named `acag`.

## Core invariant

`render_cover_svg(&CoverConfig) -> String` (in `src/cover`) is the single source
of truth. The live preview and both exports rasterize the **same SVG**, so what
you see is exactly the file you get. Never fork rendering between preview and
export: change the SVG function and both follow.

## Layout

```
src/
  design/   themes · wagara patterns · WCAG contrast
  cover/    config · format (aspect ratios) · typesetting · render · layouts
  raster.rs SVG → Pixmap/PNG by longest edge (resvg + embedded Montserrat)
  export.rs save SVG / PNG (2K or 4K)
  preset.rs save/load named presets as TOML
  wasm.rs   WebAssembly surface for the landing page (pure part tested natively)
  main.rs   Slint GUI wiring (background export + spinner)
ui/app.slint     the editor + live preview
web/             the GitHub Pages landing (index.html · css · js · generated assets)
examples/        gallery.rs (docs/samples), icon.rs (app icon + icon.ico),
                 site.rs (web/assets), build_site.rs (web/ → dist/)
build.rs         compiles the Slint UI (gui only); embeds the Windows icon resource
```

The library crate holds all logic and is what tests cover; `main.rs` is thin GUI
glue. Fonts (Montserrat Black/Bold/Regular) are embedded via `include_bytes!`, so
no system fonts are needed at runtime.

Cargo features mirror hypso's layering: `gui` (default) ⊃ `render` (resvg + file
I/O, no Slint) ⊃ the bare pure core (`cover` + `design` + `wasm`), which is what
`wasm-pack` compiles for the landing page. `export`/`preset`/`raster` only exist
under `render`; the `acag` binary requires `gui`.

## Commands

```sh
cargo run --release            # run the app
cargo test                     # unit tests
cargo fmt --all --check        # formatting gate
cargo clippy --all-targets -- -D warnings   # lint gate
cargo llvm-cov --lib --fail-under-lines 80  # coverage gate (library ≥ 80%)
cargo run --example gallery    # regenerate docs/samples
cargo run --example icon       # regenerate assets/icons/* (PNGs + icon.ico)

# Landing page (web/ → GitHub Pages)
wasm-pack build --target web --release --out-dir web/wasm --out-name acag -- --no-default-features
cargo run --example site --no-default-features --features render  # web/assets (og, poster, icons, fonts)
cargo run --example build_site --no-default-features              # assemble dist/
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy `-D warnings`, `cargo test`, and
library line coverage ≥ 80%. All four must pass; run them before proposing a
change is done.

## Conventions

- Edition 2024. Note `std::env::set_var` is `unsafe` here; only call it before any
  threads/backends start (see `main.rs`).
- Code, comments, and identifiers in English. Prefer full descriptive names, but
  keep the project's established abbreviations (`config`, `cfg`, `ctx`); match the
  surrounding code rather than renaming.
- No filler comments. Comment only non-obvious *why*, not *what*.
- Tests: cover basic functionality first, then edge cases; keep the library at
  ≥ 80% lines. Use real types, never ad-hoc untyped mocks.
- Imports inside the same directory use `crate::...` / `super::...` paths already
  established in the file; follow the surrounding style.

## Platform & runtime notes

- **Renderer**: defaults to Slint's software (CPU) renderer (`SLINT_BACKEND=winit-software`
  set in `main.rs` when unset) so it runs without OpenGL (VMs, RDP). Femtovg/GPU
  is opt-in via `SLINT_BACKEND=winit-femtovg`. Don't assume a GL context exists.
- **Windows**: `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
  hides the console for release builds. The `.exe` icon is embedded from
  `assets/icons/icon.ico` via `winresource` in `build.rs` (only on a Windows host).
- **Output dirs**: covers go to `~/Pictures/article-covers/` (Linux) /
  `%USERPROFILE%\Pictures\article-covers\` (Windows), override `ACAG_OUTPUT_DIR`.
  Presets live under the platform config dir, override `ACAG_PRESETS_DIR`.

## Release

`.github/workflows/release.yml` triggers on `v*` tags: builds a Linux AppImage +
tarball and a standalone Windows `.exe` (static CRT), attached to the GitHub
release. `.github/workflows/deploy-site.yml` publishes the landing page to
GitHub Pages on pushes to `main` that touch the site or the pure core: wasm-pack
build → `site` example → `build_site` example → deploy `dist/`. The AppImage `.desktop` `Categories` must use only freedesktop-registered
values (no unregistered `Design`, etc.) or `appimagetool` fails.

## Gotchas

- Keep preview and export pixel-identical: only the rasterization size differs.
- Regenerate `docs/samples` (`gallery`) and the icon set (`icon`) when their
  inputs change; both are committed assets. The same goes for `web/assets`
  (`site` example); only `web/wasm/` is CI-built and gitignored.
- `web/js/main.js` mirrors `poster_config()` from `examples/site.rs` as
  `FIRST_PLATE`/`FIRST_STYLE`; keep them in sync or the poster → live swap
  flickers.
- After editing `ui/app.slint`, the build re-runs `slint-build`; mismatched
  property names surface as compile errors in `main.rs`.
