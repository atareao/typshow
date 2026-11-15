---
# PLAN-009 — Botón «Salir» y atajo Ctrl+Q

- Issue: #46
- Change OpenSpec: `openspec/changes/boton-salir/`
- Estado: implementado (194 tests verdes; pendiente de PR/merge)

## Objetivo

1. Añadir un botón **«Salir»** en la barra de controles del presentador que **cierre toda la app** (presentador + audiencia).
2. Añadir el atajo **Ctrl+Q** para salir, disponible aunque se esté editando una nota.

## Contexto

- `cx.quit()` termina la app (y con ella la audiencia si proyecta). Ya se usa en `main.rs`.
- El presentador despacha teclas en `PresenterView::dispatch_key` (`src/gpui_ui/presenter.rs`, ~línea 2040) y la navegación en `keys.rs::apply_nav`.
- Los iconos son glifos de Phosphor **validados por tests** (`src/gpui_ui/icons.rs`): el nuevo icono debe existir en `assets/fonts/Phosphor.ttf`, añadirse a la constante `ALL` y a la tabla dorada del test.
- Los botones se construyen con helpers (`action_button`/`nav_button`/`control_filled`/`control_neutral`) que reciben acciones `fn(&Deck)`; «Salir» **no** es una acción de `Deck`, así que necesita un listener propio (`on_mouse_down` → `cx.quit()`).
- Decisión del usuario: el botón **cierra directamente**, sin diálogo de confirmación por cambios sin guardar.

## Alcance

### 1. `src/gpui_ui/icons.rs`
- Nuevo icono `SIGN_OUT` = `'\uE42A'` (puerta con flecha; verificado presente en `Phosphor.ttf`); incluirlo en `ALL` y en la tabla dorada del test. Verificar con `cargo test icons`.

### 2. `src/gpui_ui/presenter.rs`
- Botón «Salir» en la fila superior de la barra (junto a «Tema»), con `on_mouse_down` → método que llama a `cx.quit()`.
- `dispatch_key`: gestionar **Ctrl+Q** al principio del despacho, con independencia del modo (edición o no) → `cx.quit()` + `cx.stop_propagation()`.

### 3. `src/gpui_ui/keys.rs`
- Predicado puro (p. ej. `is_exit_shortcut(key: &str, control: bool) -> bool`) reutilizado por `presenter.rs`, con tests unitarios.

### 4. `src/gpui_ui/audience.rs`
- **Fuera de alcance**: Ctrl+Q solo se gestiona en el presentador (la audiencia se cierra vía `cx.quit()` del handler de cierre).

### 5. Tests
- `keys.rs`: Ctrl+Q → `true`; `q` sin Ctrl → `false`; Ctrl+otra tecla → `false`.
- `presenter.rs`: si es testeable en headless, comprobar que el despacho de Ctrl+Q se consume (siguiendo el patrón de los tests de `Esc`).
- Verificación manual del botón «Salir».

## No objetivos
- No hay diálogo de confirmación por cambios sin guardar.
- No se añaden otros atajos ni botones.

## Verificación
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`.
- Manual: el botón «Salir» cierra la app; Ctrl+Q cierra la app (también editando una nota).
---