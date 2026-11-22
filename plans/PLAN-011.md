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
  `load_notes_width_from` / `save_notes_width_to` (sanean al cargar y al guardar).
- Asa `handle-notas` entre `page_area` y `notes_panel`; arrastre en vivo con `cx.notify()`,
  persistencia al soltar; cursor de redimensionado.

## Tareas (TDD, espejo de tasks.md)

- [ ] 1. Constantes de límites + test.
- [ ] 2. Funciones puras de geometría + tests.
- [ ] 3. Persistencia en `config.rs` + tests (round-trip, ausente, inválido, recorte).
- [ ] 4. Campo `notes_width` y arranque desde preferencia + test.
- [ ] 5. Asa arrastrable y ancho en vivo + test.
- [ ] 6. Persistencia al soltar + test.
- [ ] 7. Auditoría, fmt/clippy/test, verificación manual, doc y archivado.

## Verificación

```bash
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Estado

- [ ] Pendiente de aprobación del spec por el usuario (fase SDD).
