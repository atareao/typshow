# Esc cierra la proyección

## Qué es

`Esc` cierra la proyección (la ventana de audiencia) siempre que esté activa, también
mientras se está editando una nota. Antes, con una edición en curso, `Esc` cancelaba la
edición y no cerraba la proyección (había que pulsar dos veces o usar el botón).

## Ficheros implicados

- `src/gpui_ui/presenter.rs` — `EscAction`, `esc_action`, `dispatch_key`.
- `src/gpui_ui/audience.rs` — handler de `Esc` de la propia ventana de audiencia (sin
  cambios; ya cerraba la proyección).

## Cómo funciona

- `esc_action(editing, projection)` decide la acción de `Esc`:
  - proyección activa → **cierra la proyección** (sea cual sea el estado de edición);
  - si no hay proyección pero hay edición → **cancela la edición**;
  - si no hay ni proyección ni edición → no hace nada.
- `dispatch_key` consulta `esc_action` para `Esc` **antes** de la rama de edición.
- Al cerrar la proyección **no** se cancela la edición: el texto que el usuario escribía se
  conserva (para descartar la edición está el botón *Cancelar*, o desactivar la proyección
  antes de pulsar `Esc`).

## Variables / configuración

- Ninguna.

## Cómo habilitar / desplegar

Es comportamiento automático; entra con el binario. No requiere migración.

## Decisiones

- La proyección tiene prioridad sobre el cancelado de edición: `Esc` cumple el requisito de
  "cerrar la proyección" sin perder el atajo de cancelar edición cuando no se proyecta.
- No se descarta la edición al cerrar la proyección (evita pérdida de datos).

## Follow-ups

- Ninguno.
