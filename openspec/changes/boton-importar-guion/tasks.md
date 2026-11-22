# Tasks

## 0. Preparación

- [ ] 0.1 Rama `feature/boton-importar-guion` desde `development`; issue #67 referenciado.

## 1. `Deck::import_guion` (RED → GREEN) — `deck.rs`

- [ ] 1.1 Test (RED): con un `.typ` cargado y un guion de tramos, importar escribe los bloques en el fichero, actualiza la fuente y reimporta las notas; devuelve el nº de bloques ⇒ `import_guion_embebe_y_reimporta`.
- [ ] 1.2 Test (RED): las notas editadas a mano se preservan y las no editadas se refrescan ⇒ `import_guion_preserva_editadas`.
- [ ] 1.3 Test (RED): un guion sin `##` ni tramos no escribe nada y avisa ⇒ `import_guion_sin_bloques_avisa`.
- [ ] 1.4 Test (RED): reimportar el mismo guion no duplica bloques ⇒ `import_guion_idempotente`.
- [ ] 1.5 Test (RED): sin documento cargado devuelve error ⇒ `import_guion_sin_documento_error`.
- [ ] 1.6 GREEN: implementar `Deck::import_guion` reutilizando `embed_script_into_typ`, `export` y `reimport_notes`; `cargo test` en verde.

## 2. Botón y diálogo (RED → GREEN) — `presenter.rs`

- [ ] 2.1 Botón `notes-importar-guion` en `notes_panel` (visible con documento), con **icono + etiqueta** como el resto.
- [ ] 2.2 Método `import_guion_dialog` con `rfd::AsyncFileDialog` filtrado a `.md` y guardia de reentrada (patrón de `open_file_dialog`).
- [ ] 2.3 Aviso **efímero** (`notes_status` con temporizador) en el panel; error si no hay documento o falla la E/S.
- [ ] 2.4 GREEN: `cargo test` + `cargo check` en verde.

## 3. Cierre

- [ ] 3.1 Auditoría `@react-…`/`@rust-reviewer`: sin hallazgos bloqueantes.
- [ ] 3.2 `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- [ ] 3.3 Verificación manual: importar un guion real (p. ej. 807) desde la app y ver las notas.
- [ ] 3.4 Doc (`docs/boton-importar-guion.md`) y archivar el change al mergear.
