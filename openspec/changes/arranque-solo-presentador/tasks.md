# Tasks

## 1. Proyección inactiva al arrancar (RED → GREEN)

- [x] 1.1 Test (RED): `Deck::empty()` devuelve `projection == false`; y un deck sin documento (`Deck::no_document(USAGE)`, antes `with_error`) también arranca con `projection == false` ⇒ `empty_starts_without_projection` y `no_document_loads_saved_theme` (`gpui_ui/deck.rs`).
- [x] 1.2 GREEN: `Deck::empty()` fija `projection: false`; `cargo test` en verde.

## 2. Tema persistente sin documento (RED → GREEN)

- [x] 2.1 Test (RED): un deck creado sin documento (`no_document_with`) usa `config::theme_path()`; `toggle_theme` persiste la preferencia y un segundo arranque la recupera ⇒ `no_document_loads_saved_theme` y `no_document_persists_theme`.
- [x] 2.2 GREEN: constructor explícito sin documento `Deck::no_document(msg)` que cablea `config::theme_path()` y persiste el tema al alternarlo; `Deck::empty()` se mantiene puro (sin ruta de tema, pensado para tests); `cargo test` en verde.

## 3. Cierre de la aplicación con el presentador (RED → GREEN)

- [x] 3.1 Test (RED): la decisión de cerrar la app vive en el predicado puro `quits_app_on_close(closing, presenter)` (`gpui_ui/mod.rs`), cubierto por tests unitarios (cierra el presentador → `true`; cierra la audiencia → `false`). El *wiring* real está cubierto headless en `gpui_ui::presenter::window_close_tests::closing_the_presenter_window_fires_the_close_handler` (`TestAppContext` + cierre de la ventana → el observer de cierre dispara el cierre de la app).
- [x] 3.2 GREEN: `main.rs` registra `App::on_window_closed` sobre la ventana del presentador y llama a `cx.quit()` cuando `quits_app_on_close` lo indica. Verificación manual pendiente en sesión gráfica (cerrar el presentador cierra también la audiencia si está proyectando).

## 4. Cierre (REFACTOR)

- [x] 4.1 `cargo fmt --all -- --check` y clippy (`--all-targets -D warnings`) limpios.
- [x] 4.2 `cargo test` en verde.
- [ ] 4.3 Crear/actualizar `docs/arranque-solo-presentador.md` y archivar el change (`openspec archive`) al mergear el issue #42.
