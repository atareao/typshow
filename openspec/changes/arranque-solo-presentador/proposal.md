# Proposal

## Why

Al abrir la app sin argumento de fichero (el caso habitual al lanzarla desde el
lanzador `.desktop`, que usa `Exec=typshow`) el comportamiento es incorrecto y
sorprendente: se abren **dos** ventanas (presentador y audiencia), hay que cerrar
las dos a mano y la preferencia de tema no se conserva. El issue #42 recoge los
tres problemas y su causa raíz.

Causa raíz: `main.rs` cae en la rama sin documento y construye el deck a partir
de `Deck::empty()` (en el código de partida, `Deck::with_error(USAGE)`; hoy el
constructor equivalente es `Deck::no_document`): `Deck::empty()` deja
`theme_path: None` (y `toggle_theme` solo persiste si el `theme_path` no es
`None`), fija `projection: true` por defecto y no hay ningún handler de cierre
de ventana que termine la aplicación.

## What Changes

- `Deck::empty()` arranca con `projection: false`, de modo que sin documento solo
  se abre la ventana del presentador.
- El tema se cablea siempre con `config::theme_path()`, incluso cuando no hay
  documento, para que la preferencia claro/oscuro se cargue y se guarde en
  cualquier arranque.
- `main.rs` registra `on_window_closed` en la ventana del presentador para que
  cerrarla invoque `cx.quit()` y termine toda la aplicación.
- El arranque sin documento se muestra como un estado del presentador (mensaje de
  uso), sin abrir la ventana de audiencia.

## Capabilities

### New Capabilities

<!-- Ninguna -->

### Modified Capabilities

- `gpui-ui`: el requirement "Toggle de proyección" pasa a exigir que la
  proyección arranque inactiva (y por tanto que sin documento solo se abra el
  presentador); el requirement "Tema claro/oscuro estilo Adwaita" pasa a exigir
  que la preferencia se cargue y se guarde también al arrancar sin documento.
  Se añade un requirement de cierre de la aplicación al cerrar el presentador.

## Non-goals

- No se cambia el formato ni la ubicación del fichero de preferencia de tema.
- No se altera la lógica de render, navegación ni notas.
- No se modifica el lanzador `.desktop` ni el empaquetado: el arreglo es en la app.
- No se introduce soporte para abrir varios documentos ni sesiones múltiples.

## Impact

- `src/main.rs` (registro de `on_window_closed` → `cx.quit()`).
- `src/gpui_ui/deck.rs` (`Deck::empty` con `projection: false`; cableado de
  `config::theme_path()` al arrancar sin documento).
- Sin cambios de API pública relevante ni de dependencias.
