# Agent Instructions: Typshow

Targeted at Linux (GNOME/Niri + DankMaterialShell), this application manages PDF-based presentations with a dual-window interface.

## Core Architecture & Intent
- **Dual Windows:**
    - **Audience Window:** Displays the current page for the audience. Opened on demand from the presenter as a fullscreen GPUI window.
    - **Presenter Window:** Displays two vertical panes. Top: current page (audience view); Bottom: next page preview. Also hosts the editable notes panel.
- **Controls:** Four primary navigation buttons (Start, Previous, Next, End), theme toggle (dark/light), and projection toggle.
- **Notes:** Markdown-based presenter notes, stored in SQLite alongside the `.typ` and editable in the presenter window.

## Technical Constraints
- **Platform:** Linux (GNOME/Niri).
- **Tooling:** Rust (2024 edition).
- **UI Stack:** `gpui` (Zed) with `gpui_platform` (Wayland/X11). One `gpui::Context` drives every window via `cx.open_window`.
- **Rendering:** Typst (`typst`, `typst-render`) compiles the `.typ` source and rasterizes pages to `RenderImage`s.
- **Fonts:** Default sans-serif (system native matches GNOME's Cantarell); icon glyphs come from the vendored Phosphor font.

## Critical Developer Commands
- `cargo build`: Standard build.
- `cargo run -- <path-to-pdf>`: Launch the application with a PDF.
- `cargo test`: Run unit and integration tests.

## Gotchas & Mistakes to Avoid
- **Multi-window Lifecycle:** Presenter and audience are separate GPUI windows sharing a single `Deck` entity; the audience window is opened/closed from the presenter.
- **DankMaterialShell:** Style tokens live in `src/gpui_ui/theme.rs` and mimic the GTK4/Adwaita aesthetic (6px corner radius, Adwaita colors, flat/active button fills).
- **State Synchronization:** Notify the `Deck` entity so both windows repaint with the current page index.
- **Wayland / Niri:** The audience window is opened as a fullscreen Wayland window with a stable `app_id`.

## Git Flow

This project follows strict gitflow. See [GIT_FLOW.md](./GIT_FLOW.md) for:
- Branch structure (main, development, feature/*, hotfix/*)
- Conventional commits with gitmoji
- How to create features, hotfixes, and releases
