# Agent Instructions: Typshow

Targeted at Linux (GNOME/Niri + DankMaterialShell), this application manages PDF-based presentations with a dual-window interface.

## Core Architecture & Intent
- **Dual Windows:**
    - **Fullscreen Window:** Displays the current PDF page for the audience. Built as an immediate viewport from `TypshowApp`.
    - **Presenter Window:** Displays two vertical panes. Top: current page (audience view); Bottom: next page preview.
- **Controls:** Four primary navigation buttons (Start, Previous, Next, End), theme toggle (dark/light), and viewport toggle.
- **Future Iteration:** Markdown-based presenter notes (to be implemented).

## Technical Constraints
- **Platform:** Linux (GNOME/Niri).
- **Tooling:** Rust (2024 edition).
- **UI Stack:** `egui` and `eframe` (v0.31). Uses multi-viewport / multi-window immediate rendering (single-threaded).
- **PDF Rendering:** `pdf-render` (pure Rust, uses `vello_cpu` and `tiny-skia` to rasterize pages).
- **Fonts:** Default sans-serif (system native matches GNOME's Cantarell).

## Critical Developer Commands
- `cargo build`: Standard build.
- `cargo run -- <path-to-pdf>`: Launch the application with a PDF.
- `cargo test`: Run unit and integration tests.

## Gotchas & Mistakes to Avoid
- **Multi-viewport Lifecycle:** Both windows are handled by a single `egui::Context` via `ctx.show_viewport_immediate`.
- **DankMaterialShell:** Style overrides in `src/theme.rs` custom-style egui visual widgets to mimic GTK4/Adwaita aesthetic (6px corner radius, Adwaita colors, flat/active button fills).
- **State Synchronization:** Keep the page index synchronized across both windows by requesting repaint in the controller window.
- **Wayland / Niri:** Fullscreen target uses Wayland viewport configuration.

## Git Flow

This project follows strict gitflow. See [GIT_FLOW.md](./GIT_FLOW.md) for:
- Branch structure (main, development, feature/*, hotfix/*)
- Conventional commits with gitmoji
- How to create features, hotfixes, and releases
