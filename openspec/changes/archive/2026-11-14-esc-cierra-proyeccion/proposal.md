# Proposal

## Why

Con una edición de notas en curso (botón *Editar* o *Insertar*), pulsar `Esc` cancela la
edición en lugar de cerrar la proyección. El presentador que está tomando notas no puede
cerrar la proyección con `Esc`: tiene que pulsar dos veces o usar el botón.

## What Changes

- `Esc` cierra la proyección **siempre que esté activa**, con independencia de que haya una
  edición de notas en curso (la proyección tiene prioridad).
- Si la proyección **no** está activa y hay una edición en curso, `Esc` sigue cancelando la
  edición (comportamiento actual).
- Si no hay proyección ni edición, `Esc` no hace nada.

## Capabilities

### New Capabilities

<!-- Ninguna -->

### Modified Capabilities

- `gpui-ui`: el requirement "Toggle de proyección" pasa a exigir que `Esc` cierre la
  proyección aunque haya una edición de notas activa.

## Impact

- `src/gpui_ui/presenter.rs` (`EscAction`, `esc_action`, `dispatch_key`).
- Sin cambios de API pública ni de dependencias.
