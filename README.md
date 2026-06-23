# Typshow

[![Version](https://img.shields.io/badge/version-0.1.5-blue)](https://github.com/atareao/typshow)
[![Rust](https://img.shields.io/badge/rust-2024-edition?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux-lightgrey?logo=linux)](.)

**Dual-window Typst presentation tool for Linux (GNOME/Niri).**

Typshow displays Typst slide decks on a fullscreen projector while you control navigation from a separate presenter window with page previews and editable notes.

## Features

- **Dual-window interface** — presenter controls + fullscreen audience projection
- **Typst-native** — write slides in [Typst](https://typst.app/) markup (`.typ`); the title and headings are extracted automatically
- **Editable notes** — SQLite-backed presenter notes per page, created from your `.typ` source structure
  - Each note stores a heading (from Typst `=`, `==`, `===` sections) and content
  - Edit notes inline with Markdown formatting support
  - **Insert** a new note, **delete** current note, or **reimport** from the original `.typ` source
  - Notes persist in a `nome.typshow.db` file alongside your `.typ` file
- **Live preview** — presenter window shows current + next slide
- **Dark/light theme** — Adwaita-inspired GTK4 style for both modes
- **Keyboard navigation** — arrow keys, Page Up/Down, Vim-style keys (J/K/H/L), mouse clicks
- **Niri/Wayland support** — auto-moves projection window to secondary monitor

## Usage

```bash
# Launch with a Typst document
cargo run -- path/to/presentation.typ

# Launch empty (open file from UI)
cargo run
```

### Controls

**Presenter window:**

| Key | Action |
|---|---|
| `Right Arrow` / `Page Down` / `J` | Next slide |
| `Left Arrow` / `Page Up` / `K` | Previous slide |
| `H` | First slide |
| `L` | Last slide |
| `Escape` | Close projection |

All keyboard shortcuts are disabled while editing a note.

**Projection window:**

| Input | Action |
|---|---|
| `J` / `Right Arrow` / `Page Down` | Next slide |
| `K` / `Left Arrow` / `Page Up` | Previous slide |
| `H` | First slide |
| `L` | Last slide |
| `Escape` | Close projection |
| Left-click | Previous slide |
| Right-click | Next slide |

### Notes panel

The right sidebar shows notes for the current slide:

| Button | Action |
|---|---|
| ✏️ Editar nota | Open note for editing |
| 💾 Guardar | Save current note |
| ❌ Cancelar | Discard edits |
| ➕ Nota nueva | Insert a new note (duplicates and shifts following pages) |
| 🗑 Eliminar nota | Delete current note |
| 🔄 Reimportar | Re-import notes from the original `.typ` source |

The heading is also editable — click the ✏️ next to the heading text.

Notes auto-save when navigating to another page if edits are dirty.

## Architecture

```
main.rs ── AppState (Arc<Mutex>) ── TypshowApp (presenter window)
                                       └── FullscreenApp (projection via show_viewport_immediate)

Renderer ── synchronous rendering with texture cache (HashMap<usize, TextureHandle>)
             └── typ → typst-compile → typst-render → tiny-skia

Notes ── SQLite (rusqlite) via NotesDb ── notes.db file alongside .typ
          └── CRUD: page_number (PK), heading, content, updated_at
```

Both windows share a single `egui::Context` via multi-viewport immediate mode. The renderer compiles Typst sources synchronously and delivers textures to a `HashMap<usize, TextureHandle>` cache.

### Notes database format

Each `.typ` file has a companion SQLite database (`nome.typshow.db`) with schema:

```sql
CREATE TABLE notes (
    page_number INTEGER PRIMARY KEY,
    heading     TEXT,
    content     TEXT NOT NULL DEFAULT '',
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

On first open, the database is populated from the `.typ` source:
- Page 0 gets its heading from `title:` in the `slides.with(...)` call
- `=` and `==` headings get exactly one page each
- `===` headings and deeper distribute remaining pages evenly
- Page breaks (`#pagebreak()`) create new segment boundaries

## Emoji support

For emoji rendering (📂🖥✏️💾❌), you need a monochrome emoji font:

- **Arch Linux:** `paru -S ttf-noto-emoji-monochrome`
- **Other distros:** Install `Noto Emoji` (the monochrome version, not `Noto Color Emoji`)

## Requirements

- **Linux** with Wayland (GNOME or Niri)
- **System packages:** `libasound2-dev`, `pkg-config`, `libwayland-dev`, `libxkbcommon-dev`
- **Rust** 2024 edition (`cargo`)

## Build

```bash
cargo build --release
./target/release/typshow path/to/deck.typ
```

## Development

```bash
cargo test              # run tests (61 unit + 2 integration)
cargo fmt --all --check # check formatting
cargo clippy            # lint
```

## Project status

Active development (v0.1.5). The core viewing, navigation, and SQLite-powered notes system are functional. See [GIT_FLOW.md](./GIT_FLOW.md) for branch conventions and commit format.