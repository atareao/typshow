# Design

## Context

`PresenterView::dispatch_key` (`src/gpui_ui/presenter.rs`) decide en dos ramas: si hay un
editor activo (`active_target().is_some()`), toda tecla va a `handle_edit_key` (donde `Esc`
= `EditorCommand::Cancel`); si no, `Esc` cierra la proyección vía `esc_action`. Por eso, con
una edición en curso, `Esc` nunca llega a cerrar la proyección.

## Goals / Non-Goals

**Goals:**
- `Esc` cierra la proyección siempre que esté activa, también durante la edición.

**Non-Goals:**
- Cambiar cómo se cancela una edición cuando no hay proyección.

## Decisions

- **`EscAction` gana la variante `CancelEdit`** y `esc_action(editing, projection)` pasa a ser
  la fuente única de la precedencia: `projection` → `CloseProjection`; si no, `editing` →
  `CancelEdit`; si no, `None`.
- **`dispatch_key` consulta `esc_action` para la tecla `Esc` antes de la rama de edición.**
  Con `CloseProjection` cierra la proyección y corta la propagación; con `CancelEdit` delega
  en `handle_edit_key` (que ya mapea `Esc` a `Cancel`); con `None` no hace nada.
- **La edición permanece abierta al cerrar la proyección**: `Esc` no descarta el texto que el
  usuario escribía (evita pérdida de datos). Para cancelar la edición está el botón
  *Cancelar*.

## Risks / Trade-offs

- Con la proyección activa, `Esc` ya no cancela la edición; el usuario debe usar *Cancelar*
  (o desactivar la proyección). Es el comportamiento pedido.
