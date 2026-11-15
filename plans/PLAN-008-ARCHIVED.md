---
# PLAN-008 — Arranque solo con el presentador, cierre conjunto y persistencia del tema

- Issue: (referenciado por el subagente que crea el issue; ver descripción de la feature)
- Change OpenSpec: `openspec/changes/arranque-solo-presentador/`
- Estado: propuesto (pendiente de aprobación del usuario antes de escribir código)

## Objetivo

1. Al abrir la app debe abrirse **solo** la ventana del presentador (hoy se abren presentador + audiencia).
2. Al cerrar la app debe cerrarse **todo** (presentador + audiencia) con una sola acción de cierre.
3. La preferencia de tema (claro/oscuro) debe persistir entre sesiones **aunque la app se arranque sin fichero `.typ`**.

## Diagnóstico (evidencia)

- El lanzador `~/.local/share/applications/typshow.desktop` usa `Exec=typshow` (sin argumento) → `main.rs` crea `Deck::no_document(USAGE)` → `Deck::empty()` → `theme_path: None`.
- `Deck::toggle_theme` (src/gpui_ui/deck.rs) solo persiste si `theme_path` es `Some`; sin documento nunca es `Some`, así que el tema **no se guarda**.
  - Verificado: con argumento `Deck::load` escribe `~/.config/typshow/theme`; arrancando sin argumento no se escribe nada (el directorio ni existía).
- `Deck::empty()` fija `projection: true` (deck.rs:69) → `PresenterView::new` abre la ventana de audiencia (presenter.rs:1279) al arrancar.
- No existe handler que cierre la app al cerrar la ventana del presentador; con la audiencia abierta, cerrar el presentador deja la audiencia viva.

## Alcance / cambios

### 1. `src/gpui_ui/deck.rs`
- `Deck::empty()`: cambiar `projection: true` → `projection: false`.
- Cablear el tema **siempre**, con independencia del documento cargado:
  - Mantener `empty()` "puro" (`theme_path: None`) para los tests.
  - Nuevo constructor usado por la app cuando NO hay documento (p. ej. `Deck::no_document(msg)`): fija `theme_path = config::theme_path()`, aplica `dark` desde disco y luego pone el mensaje de error.
  - `Deck::load`/`load_with` conservan el cableado actual del tema.
  - Extraer un helper privado para "aplicar el tema guardado" y reutilizarlo en los tres caminos.

### 2. `src/main.rs`
- La rama sin documento debe usar el nuevo constructor `Deck::no_document` (tema cableado), no `Deck::empty()` sin tema.
- Tras abrir la ventana del presentador, registrar `cx.on_window_closed` de modo que, si el `window_id` que se cierra es el del presentador, se llame a `cx.quit()` (esto cierra también la audiencia y termina la app).

### 3. Tests
- `deck.rs`: `Deck::empty().projection == false`.
- `deck.rs`: el constructor sin documento carga el tema guardado (inyectando la ruta) y lo persiste al alternarlo.
- `presenter.rs`: (si es viable en headless con `TestAppContext`) cerrar el presentador termina la app; si no es viable, documentarlo y cubrir la lógica en `deck`/`main`.
- Revisar que ningún test existente asuma `projection == true` por defecto.

## No objetivos
- No cambiar el flujo manual de "Mostrar/Ocultar proyección".
- No tocar notas ni SQLite.
- No cambiar el comportamiento de "cerrar la ventana de audiencia" (sigue desactivando la proyección).

## Verificación
- `cargo test`
- `cargo clippy --all-targets -- -D warnings` (CI usa 1.100.0: `rustup run 1.100.0 cargo clippy --all-targets -- -D warnings`)
- `cargo fmt --check`
- Manual: lanzar `typshow` sin argumentos → solo presentador; alternar tema → se crea `~/.config/typshow/theme`; relanzar → conserva el tema; cerrar el presentador → se cierra también la audiencia.
---