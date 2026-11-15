# Proposal

## Why

Hoy no hay ninguna forma explícita de terminar Typshow desde la interfaz: la
única manera es cerrar la ventana del presentador con el gestor de ventanas. No
existe un botón de salida ni el atajo estándar `Ctrl+Q`, que los usuarios de
escritorio GNOME/Adwaita esperan encontrar. El issue #46 recoge la petición.

Además, el editor de notas captura la mayoría de teclas mientras hay una edición
en curso (`dispatch_key` desvía los eventos al editor), así que un atajo de salida
debe decidirse en un punto que no quede secuestrado por el editor.

## What Changes

- Se añade un botón **«Salir»** en la barra superior de controles del presentador,
  junto al botón «Tema» (`btn-tema`).
- Se añade el atajo **`Ctrl+Q`**, activo tanto en modo normal como durante la
  edición de una nota.
- Ambos caminos llaman a un mismo `PresenterView::exit_app`, que cierra la ventana
  del presentador (`window.remove_window()`, lo que notifica el cierre y desactiva la
  proyección) y pide el fin de la app (`cx.quit()`), sin depender del handler de
  `main.rs`. En la práctica termina presentador y audiencia a la vez.
- El cierre es **directo**: sin diálogo de confirmación y sin tratar de forma
  especial los cambios sin guardar.
- Se añade un icono Phosphor de salida (con codepoint válido, incluido en `ALL` y
  en la tabla dorada del test de `src/gpui_ui/icons.rs`).

## Capabilities

### New Capabilities

<!-- Ninguna -->

### Modified Capabilities

- `gpui-ui`: se añade el requirement "Salida de la aplicación", que exige el
  botón «Salir» en la barra del presentador y el atajo `Ctrl+Q` —también durante
  la edición de una nota—, ambos terminando la app al completo.

## Non-goals

- No se añaden más atajos de teclado ni combinaciones alternativas.
- No se introduce diálogo de confirmación ni aviso de cambios sin guardar.
- No se cambia el comportamiento de cierre al cerrar la ventana del presentador
  (`on_window_closed` → `cx.quit()`, ya existente desde el issue #42).
- No se altera la navegación, la proyección, el tema ni el modelo de notas.
- No se modifica el lanzador `.desktop` ni el empaquetado.

## Impact

- `src/gpui_ui/keys.rs`: predicado puro del atajo de salida (p. ej.
  `quit_shortcut(key, control)`) más sus tests unitarios.
- `src/gpui_ui/icons.rs`: nuevo icono de salida, incluido en `ALL` y en la tabla
  dorada; el test de la fuente embebida lo valida automáticamente.
- `src/gpui_ui/presenter.rs`: botón «Salir» en la barra superior (listener propio
  porque la acción no es de `Deck`) y rama de `Ctrl+Q` en `dispatch_key`, previa a
  la bifurcación edición/navegación.
- Sin cambios de API pública relevante ni de dependencias.
