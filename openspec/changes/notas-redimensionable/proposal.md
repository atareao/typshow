# Proposal

## Why

En la ventana del presentador, el panel de «Notas» es una barra lateral de **ancho fijo**
(`NOTES_PANEL_WIDTH = 320.` en `src/gpui_ui/theme.rs`), hermana de la previsualización en
`main_row` (`src/gpui_ui/presenter.rs`). El ancho no se puede ajustar: en pantallas
pequeñas roba espacio a las páginas y en pantallas grandes desperdicia sitio que el
presentador agradecería para leer notas largas.

El issue #55 pide poder **redimensionar ese panel** arrastrando el separador entre el panel
y la previsualización, con un **ancho mínimo y máximo**, y que el ancho elegido **persista
entre sesiones** — igual que ya persiste el tema (fichero `<config_dir>/theme` vía
`src/gpui_ui/config.rs`).

## What Changes

- La anchura del panel de notas pasa a ser estado: `PresenterView.notes_width` (f32),
  inicializado desde las preferencias y con `NOTES_PANEL_WIDTH` como valor por defecto.
- Se añaden **límites** (`NOTES_PANEL_MIN_WIDTH`, `NOTES_PANEL_MAX_WIDTH`) y una función
  pura de recorte (`clamp_notes_width`) reutilizable por UI y persistencia.
- Se añade un **asa arrastrable** (`handle-notas`) entre la previsualización y el panel:
  al arrastrar se recalcula el ancho en vivo (`notes_width_after_drag(start, dx)`), y al
  soltar se **persiste**.
- La **persistencia** replica el patrón del tema en `src/gpui_ui/config.rs`:
  `load_notes_width_from` / `save_notes_width_to` sobre `<config_dir>/notes_width`: los
  valores ausentes, no numéricos o **fuera de rango** ⇒ ancho por defecto al cargar; el
  guardado recorta (sanea) al rango válido antes de escribir.
- **Feedback de cursor** de redimensionado mientras se arrastra el asa.

## Capabilities

### New Capabilities

- `gpui-ui`: se amplía con el requisito «Panel de notas redimensionable» (arrastre,
  límites, persistencia).

### Modified Capabilities

- Ninguna. El render de páginas, la audiencia y la edición de notas no cambian.

## Impact

- `src/gpui_ui/presenter.rs`: campo `notes_width` y estado de arrastre en `PresenterView`;
  asa `handle-notas`; `notes_panel` usa el ancho de estado.
- `src/gpui_ui/theme.rs`: constantes `NOTES_PANEL_MIN_WIDTH` / `NOTES_PANEL_MAX_WIDTH`.
- `src/gpui_ui/config.rs`: carga/guardado del ancho (patrón del tema).
- Sin cambios de esquema SQLite ni del `.typ`. Issue: #55.
