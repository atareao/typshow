# Typshow

[![Version](https://img.shields.io/badge/version-0.1.0-blue)](https://github.com/atareao/typshow)
[![Rust](https://img.shields.io/badge/rust-2024-edition?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux-lightgrey?logo=linux)](.)

**Dual-window PDF & Typst presentation tool for Linux.**

Typshow displays slide decks on a fullscreen projector while you control navigation from a separate presenter window with page previews.

## Features

- **Dual-window interface** — presenter controls + fullscreen audience projection
- **PDF & Typst support** — open `.pdf` files or write slides in [Typst](https://typst.app/) markup (`.typ`)
- **Live preview** — presenter window shows current + next slide
- **Background rendering** — pages render asynchronously with texture caching
- **Dark/light theme** — Adwaita-inspired GTK4 style for both modes
- **Keyboard navigation** — arrow keys, Page Up/Down in presenter window; Vim-style keys (J/K/H/L) plus mouse clicks in projection
- **Niri/Wayland support** — auto-moves projection window to secondary monitor

## Usage

```bash
# Launch with a document
cargo run -- path/to/presentation.pdf

# Launch with Typst source
cargo run -- path/to/slides.typ

# Launch empty (open file from UI)
cargo run
```

### Controls

**Presenter window:**

| Key | Action |
|---|---|
| `Right Arrow` / `Page Down` | Next slide |
| `Left Arrow` / `Page Up` | Previous slide |

**Projection window:**

| Key | Action |
|---|---|
| `J` / `Right Arrow` / `Page Down` | Next slide |
| `K` / `Left Arrow` / `Page Up` | Previous slide |
| `H` | First slide |
| `L` | Last slide |
| `Escape` | Close projection |

> Tip: Left-click on the projected slide goes back; right-click goes forward.

## Architecture

```
main.rs ── AppState (Arc<Mutex>) ── TypshowApp (presenter window)
                                       └── FullscreenApp (projection via show_viewport_immediate)

PdfRenderer ── background thread with command/result channels
                ├── PDF path: pdf → pdf-render → tiny-skia
                └── Typst path: typ → typst-compile → typst-render → vello
```

Both windows share a single `egui::Context` via multi-viewport immediate mode. The render thread processes pages asynchronously and delivers textures to a `HashMap<usize, TextureHandle>` cache.

## Requirements

- **Linux** with Wayland (GNOME or Niri)
- **System packages:** `libasound2-dev`, `pkg-config`, `libwayland-dev`, `libxkbcommon-dev`
- **Rust** 2024 edition (`cargo`)

## Build

```bash
cargo build --release
./target/release/typshow path/to/deck.pdf
```

## Development

```bash
cargo test              # run tests
cargo fmt --all --check # check formatting
cargo clippy            # lint
```

## Keyboard shortcuts

See full reference in [GIT_FLOW.md](./GIT_FLOW.md) for branch conventions and commit format.

## Project status

Early development (v0.1.0). The core viewing and navigation is functional. Future work includes presenter notes support (Markdown-based).