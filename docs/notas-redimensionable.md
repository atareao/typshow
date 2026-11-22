# Panel de notas redimensionable

## Qué es

La barra lateral de notas del presentador se puede **redimensionar en ancho**
arrastrando el asa `handle-notas`, situada entre la previsualización y el panel.
El ancho queda acotado entre `NOTES_PANEL_MIN_WIDTH` (200 px) y
`NOTES_PANEL_MAX_WIDTH` (640 px) y **persiste entre sesiones**.

Issue: #55 · OpenSpec: `openspec/changes/notas-redimensionable/` · Plan: `plans/PLAN-011.md`.

## Ficheros implicados

- `src/gpui_ui/theme.rs` — `NOTES_PANEL_WIDTH` (320, por defecto), `NOTES_PANEL_MIN_WIDTH`,
  `NOTES_PANEL_MAX_WIDTH`.
- `src/gpui_ui/config.rs` — persistencia del ancho (`<config_dir>/notes_width`).
- `src/gpui_ui/presenter.rs` — estado del ancho, asa arrastrable y geometría pura.

## Cómo funciona

- **Estado**: `PresenterView.notes_width` (f32). El panel pinta `.w(px(self.notes_panel_width()))`,
  donde `notes_panel_width()` recorta el estado a los límites.
- **Arranque**: `PresenterView::new` inyecta `config::notes_width_path()` en
  `PresenterView::new_with`, que resuelve el ancho con `initial_notes_width(preference)`: la
  preferencia leída o `NOTES_PANEL_WIDTH` (320) si el fichero no existe, su contenido no es un
  número finito o el valor cae **fuera de rango**. Al cargar los valores fuera de rango se
  **descartan** (⇒ ancho por defecto); el recorte es cosa del guardado.
- **Geometría pura** (sin GPUI): `clamp_notes_width(w)` recorta a `[min, max]`;
  `notes_width_after_drag(start, dx)` devuelve `clamp_notes_width(start - dx)`. El asa está a la
  derecha del área de página, así que arrastrar hacia la izquierda (`dx < 0`) **aumenta** el ancho.
- **Arrastre**: `on_mouse_down` del asa llama a `begin_notes_drag` (guarda la coordenada X del
  puntero y el ancho de partida); `on_mouse_move` de la fila principal (`main_row`) llama a
  `drag_notes`, que aplica el desplazamiento y hace `cx.notify()` solo si el ancho cambia. Se
  registra en la fila y no solo en el asa porque el asa mide 6 px y el puntero sale de ella en
  cuanto se mueve.
- **Persistencia**: `on_mouse_up` (y `on_mouse_up_out`, para soltar fuera del asa) llaman a
  `end_notes_drag`, que persiste con `persist_notes_width` → `config::save_notes_width_to`
  (recorta antes de escribir y crea el directorio padre). Es best-effort: un fallo de E/S no
  rompe la sesión.
- **Cursor**: el asa usa `CursorStyle::ResizeLeftRight` (CSS `ew-resize`) al pasar por encima; y
  mientras `notes_dragging` está activo, `main_row` impone el mismo cursor (`row_cursor`) para que
  no revierta cuando el puntero sale del asa de 6 px.

## Variables / configuración

- `XDG_CONFIG_HOME` (opcional): si es una ruta absoluta, se usa como base de configuración; si no,
  `$HOME/.config`. Fichero: `<config_dir>/typshow/notes_width` con el ancho en una línea.
- Sin preferencia válida, el ancho por defecto es 320 px.

## Cómo habilitar / desplegar

Es comportamiento automático de la ventana del presentador; no requiere migración ni flags.

## Decisiones

- Sin dependencias nuevas: la ruta se resuelve con `std` (mismo patrón que la preferencia de tema).
- Asimetría deliberada: al **cargar** un valor fuera de rango se descarta (⇒ ancho por defecto);
  al **guardar** se recorta al rango. Así un fichero editado a mano no deja el panel fuera de
  rango, y la preferencia nunca se corrige «en silencio» a un ancho que el usuario no eligió.
- La ruta de la preferencia se inyecta en el constructor (`new_with`) para que los tests usen un
  directorio temporal sin tocar el entorno.
- El cursor de redimensionado horizontal existe en la rev de GPUI pinneada
  (`CursorStyle::ResizeLeftRight`), así que no hizo falta un cursor alternativo.
- Un fallo de E/S al persistir se registra con `warn!` y no interrumpe el arrastre.

## Follow-ups

- Verificación manual en GUI (arrastrar el asa, comprobar límites y persistencia tras reiniciar):
  tarea 7.3 de `tasks.md`.
- Archivar el change de OpenSpec al mergear (tarea 7.4).

## Tests

`cargo test` cubre: `limites_ancho_notas` (theme); `round_trip_ancho_notas`,
`ancho_notas_ausente_none`, `ancho_notas_invalido_none`, `ancho_notas_fuera_de_rango_none`,
`ancho_notas_recorta_y_crea_dir` (config); `clamp_notas_min_max`, `ancho_tras_arrastre`,
`arranque_usa_preferencia_o_default`, `panel_usa_ancho_de_estado`, `row_cursor_solo_durante_el_arrastre`,
`soltar_persiste` y `arrastre_en_vivo_persiste_al_soltar` (presenter).
