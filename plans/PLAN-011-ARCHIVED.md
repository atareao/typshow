# PLAN-011 — Panel de notas redimensionable

- **Issue:** #55 · **Rama:** `feature/notas-redimensionable` (base `development`) · **OpenSpec:** `openspec/changes/notas-redimensionable/`

## Objetivo

Permitir redimensionar el ancho de la barra lateral de notas del presentador
arrastrando un asa, con límites mínimo/máximo y persistencia entre sesiones.

## Diseño

- Anchura como estado: `PresenterView.notes_width` (f32); defecto `NOTES_PANEL_WIDTH` (320).
- Límites en `theme.rs`: `NOTES_PANEL_MIN_WIDTH = 200.`, `NOTES_PANEL_MAX_WIDTH = 640.`.
- Lógica pura testeable: `clamp_notes_width(w)` y `notes_width_after_drag(start, dx)`.
- Persistencia patrón-tema en `config.rs`: `<config_dir>/notes_width`,
  `load_notes_width_from` (descarta ausente/no numérico/no finito/**fuera de rango** ⇒ `None`)
  / `save_notes_width_to` (recorta al rango antes de escribir).
- Asa `handle-notas` entre `page_area` y `notes_panel`; arrastre en vivo con `cx.notify()`,
  persistencia al soltar; cursor de redimensionado.

## Tareas (TDD, espejo de tasks.md)

- [x] 1. Constantes de límites + test.
- [x] 2. Funciones puras de geometría + tests.
- [x] 3. Persistencia en `config.rs` + tests (round-trip, ausente, inválido, recorte).
- [x] 4. Campo `notes_width` y arranque desde preferencia + test.
- [x] 5. Asa arrastrable y ancho en vivo + test.
- [x] 6. Persistencia al soltar + test.
- [ ] 7. Auditoría, fmt/clippy/test, verificación manual, doc y archivado.
  - [x] 7.2 `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test` en verde.
  - [x] 7.1 Auditoría `@rust-reviewer` (APROBADA) · [x] 7.3 Verificación manual (GUI) · [x] 7.4 Doc + archivado al mergear (PR #56).

## Implementación (referencias)

- `src/gpui_ui/theme.rs`: `NOTES_PANEL_MIN_WIDTH`, `NOTES_PANEL_MAX_WIDTH`; test `limites_ancho_notas`.
- `src/gpui_ui/config.rs`: `NOTES_WIDTH_FILE`, `notes_width_path`, `load_notes_width_from`,
  `notes_width_in_range`, `save_notes_width_to`, `sanitize_notes_width`; tests
  `round_trip_ancho_notas`, `ancho_notas_ausente_none`, `ancho_notas_invalido_none`,
  `ancho_notas_fuera_de_rango_none`, `ancho_notas_recorta_y_crea_dir`.
- `src/gpui_ui/presenter.rs`: `clamp_notes_width`, `notes_width_after_drag`,
  `initial_notes_width`, `row_cursor`, `persist_notes_width`;
  `PresenterView::{notes_panel_width, notes_handle, begin_notes_drag, drag_notes, end_notes_drag}`;
  constructor `new_with` (ruta de preferencia inyectable); tests en `mod notes_width_tests`.

## Verificación

```bash
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Estado

- [x] Implementación completa (1–6) y verificación estática/test (7.2).
- [x] COMPLETADA: implementación, verificación estática/test, auditoría aprobada, verificación manual en GUI y archivado (merge PR #56, `55e1cbc`).
