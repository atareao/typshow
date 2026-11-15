# Tasks

## 1. Predicado puro del atajo de salida (RED → GREEN)

- [x] 1.1 Test (RED): el predicado `quit_shortcut(key, control)` (`gpui_ui/keys.rs`) devuelve `true` para `Ctrl+Q` (`"q"` con `control == true`) y `false` para `q` sin modificador, para `Ctrl+` otras teclas y para otros modificadores ⇒ `quit_shortcut_matches_ctrl_q`.
- [x] 1.2 GREEN: implementar `quit_shortcut` como función pura en `keys.rs` (sin dependencias de GPUI más allá del contrato existente); `cargo test` en verde.

## 2. Icono de salida (RED → GREEN)

- [x] 2.1 Test (RED): ampliar la tabla dorada de `los_codepoints_coinciden_con_la_tabla_dorada` con la nueva constante (nombre, valor y codepoint documentado) y el tamaño de `ALL` (de 16 a 17); el test de `ALL` sin repetidos y el de glifo presente en la fuente cubren el resto ⇒ la tabla falla hasta añadir la constante.
- [x] 2.2 GREEN: definir la constante del icono Phosphor de salida (p. ej. `SIGN_OUT`) en `src/gpui_ui/icons.rs`, añadirla a `ALL` y a la tabla dorada del test; `cargo test` en verde.

## 3. Botón «Salir» en la barra del presentador (RED → GREEN)

- [x] 3.1 Test (RED): el wiring real se cubre headless con `TestAppContext` en un test de `gpui_ui::presenter` (mismo patrón que `window_close_tests`): pulsar el botón «Salir» dispara `cx.quit()` (el handler de cierre), con o sin proyección activa.
- [x] 3.2 GREEN: añadir el control `btn-salir` inmediatamente después de `btn-tema` en la barra superior, con `icons::label(icons::SIGN_OUT, "Salir")` y un `cx.listener` propio que llame a `exit_app` (que hace `window.remove_window()` + `cx.quit()`; la acción no es de `Deck`); `cargo test` en verde.

## 4. Atajo Ctrl+Q en `dispatch_key` (RED → GREEN)

- [x] 4.1 Test (RED): test headless de `PresenterView` que envía `Ctrl+Q` y comprueba que se dispara el cierre de la app, tanto sin edición como **con una edición de nota en curso** (el evento MUST NOT llegar al editor).
- [x] 4.2 GREEN: en `PresenterView::dispatch_key`, comprobar `quit_shortcut(keystroke.key, keystroke.modifiers.control)` **antes** de la bifurcación `editing`/navegación y, si coincide, llamar a `exit_app` y `cx.stop_propagation()`; el resto de teclas mantiene el flujo actual ⇒ `cargo test` en verde.

## 5. Verificación y cierre (REFACTOR)

- [x] 5.1 `cargo fmt --all -- --check` y clippy (`--all-targets -D warnings`) limpios.
- [x] 5.2 `cargo test` en verde (sin regresiones en los tests de `keys`, `icons` y `presenter`).
- [x] 5.3 Verificación manual en sesión gráfica: «Salir» cierra toda la app; `Ctrl+Q` cierra la app con y sin edición de nota en curso.
- [ ] 5.4 Crear/actualizar `docs/boton-salir.md` y archivar el change (`openspec archive`) al mergear el issue #46.
