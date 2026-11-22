# Proposal

## Why

El guion solo puede embeberse en el `.typ` por CLI (`typshow guion-embed`). Desde el
presentador no hay forma de importar el guion: el presentador ya muestra y edita las notas
por página, pero el guion hay que meterlo a mano por terminal. Un botón en el panel de notas
que haga el embebido y refresque las notas cierra el circuito dentro de la app. Issue #67.

## What Changes

- **Botón «Importar guion»** en el panel de notas del presentador (junto a Guardar / Nueva /
  Borrar), visible cuando hay un `.typ` cargado.
- Al pulsarlo, abre un diálogo `rfd` filtrado a `.md` (mismo patrón que «Abrir»).
- Con el guion elegido, el `Deck`:
  1. lee el guion,
  2. ejecuta el embebido existente (`embed_script_into_typ`: modo por rango #59 o por
     secciones `##` #52),
  3. escribe los bloques en el fichero del `.typ` abierto,
  4. actualiza su fuente y **reimporta las notas** con la reimportación selectiva por hash
     (refresca las no editadas y **preserva** las editadas a mano),
  5. devuelve cuántos bloques se importaron.
- **Feedback**: mensaje con el nº de bloques importados, o aviso/error (guion sin secciones
  ni tramos, sin documento cargado, error de E/S).

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `gpui-ui`: nuevo requisito «Botón «Importar guion» en el panel de notas».
- `notas`: nuevo requisito «Importación del guion desde la aplicación» (embebido + escritura
  + reimportación selectiva).

## Impact

- `src/gpui_ui/presenter.rs`: botón en `notes_panel` + método `import_guion_dialog` (patrón
  de `open_file_dialog`), estado del diálogo.
- `src/gpui_ui/deck.rs`: `Deck::import_guion(path) -> Result<usize, String>`.
- Sin cambios en el mapeo, en el CLI (`guion-embed` se mantiene) ni en el esquema SQLite.
