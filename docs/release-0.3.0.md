# Release 0.3.0

Fecha: 2026-11-14 · Tipo: minor · Tag: `v0.3.0`

## Qué incluye

- ✨ **feat(gpui)**: persiste la preferencia de tema (claro/oscuro) entre sesiones.
  Nuevo `src/gpui_ui/config.rs` (XDG `~/.config/typshow/theme`, sin dependencias). `Deck`
  gana `theme_path`, lo carga en `load` (inyectable con `load_with`) y lo guarda en
  `toggle_theme`. Issue #35 · PR #36.
- 🐛 **fix(gpui)**: renderiza la cursiva en el markdown de las notas.
  Nuevo `InlineStyle::Italic` y rama `*…*` en `inline_runs` (precedencia `**` → `` ` `` → `*`,
  con guarda para marcadores sin cerrar); `inline_text` aplica `FontStyle::Italic`.
  `notes::parse_md` pasa a `pub(crate)`. Issue #33 · PR #37.
- 🐛 **fix(gpui)**: `Esc` cierra la proyección aunque se esté editando.
  Nuevo `EscAction::CancelEdit`; `esc_action` da prioridad a la proyección y `dispatch_key`
  la consulta antes de la rama de edición. La edición permanece abierta al cerrar la
  proyección. Issue #34 · PR #38.
- 📝 **docs(openspec)**: archiva los changes `persistir-tema`, `markdown-cursiva` y
  `esc-cierra-proyeccion`. PR #39.

## Verificación

- Tests en verde; `cargo fmt` y clippy 1.100.0 (`-D warnings`) limpios.
- Verificación visual en Niri **pendiente**: tema persistente tras reinicio, cursiva en las
  notas y `Esc` cierra la proyección con el editor abierto.

## Artefactos

- `typshow-x86_64-unknown-linux-gnu.tar.gz`
- `typshow-aarch64-unknown-linux-gnu.tar.gz`

## Limitaciones conocidas

- crates.io: no publicable mientras `gpui`/`gpui_platform` sean dependencias git de Zed. El
  job `publish` es no bloqueante (falla con exit 101, esperado).
