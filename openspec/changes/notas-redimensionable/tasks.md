# Tasks

## 0. Preparación

- [ ] 0.1 Rama `feature/notas-redimensionable` desde `development`; issue #55 referenciado.

## 1. Límites y recorte del ancho (RED → GREEN) — `theme.rs`

- [ ] 1.1 Test (RED): `NOTES_PANEL_MIN_WIDTH < NOTES_PANEL_WIDTH < NOTES_PANEL_MAX_WIDTH` y valores esperados ⇒ `limites_ancho_notas`.
- [ ] 1.2 GREEN: declarar `NOTES_PANEL_MIN_WIDTH` (200.) y `NOTES_PANEL_MAX_WIDTH` (640.) en `theme.rs`; `cargo test` en verde.

## 2. Función pura de geometría (RED → GREEN) — `presenter.rs`

- [ ] 2.1 Test (RED): `clamp_notes_width` recorta por debajo del mínimo y por encima del máximo, y deja pasar los intermedios ⇒ `clamp_notas_min_max`.
- [ ] 2.2 Test (RED): `notes_width_after_drag(start, dx)` aplica `start - dx` y recorta ⇒ `ancho_tras_arrastre`.
- [ ] 2.3 GREEN: implementar ambas funciones puras; `cargo test` en verde.

## 3. Persistencia del ancho (RED → GREEN) — `config.rs`

- [ ] 3.1 Test (RED): `save_notes_width_to`/`load_notes_width_from` hacen round-trip de un valor válido ⇒ `round_trip_ancho_notas`.
- [ ] 3.2 Test (RED): fichero ausente ⇒ `None` ⇒ `ancho_notas_ausente_none`.
- [ ] 3.3 Test (RED): contenido no numérico ⇒ `None` ⇒ `ancho_notas_invalido_none`.
- [ ] 3.4 Test (RED): valor fuera de rango (alto/bajo) se recorta al cargar ⇒ `ancho_notas_recortado_al_cargar`.
- [ ] 3.5 Test (RED): `save_notes_width_to` recorta antes de escribir y crea el directorio padre ⇒ `ancho_notas_recorta_y_crea_dir`.
- [ ] 3.6 GREEN: `NOTES_WIDTH_FILE`, `notes_width_path`, `load_notes_width_from`, `save_notes_width_to`; `cargo test` en verde.

## 4. Estado y arranque (RED → GREEN) — `presenter.rs`

- [ ] 4.1 Test (RED): `PresenterView` inicializa `notes_width` desde la preferencia cuando existe y desde `NOTES_PANEL_WIDTH` cuando no ⇒ `arranque_usa_preferencia_o_default`.
- [ ] 4.2 GREEN: campo `notes_width` + estado de arrastre en `PresenterView`; init en el constructor; `cargo test` en verde.

## 5. Asa arrastrable y aplicación en vivo (RED → GREEN) — `presenter.rs`

- [ ] 5.1 Test (RED): `notes_panel` usa `self.notes_width` (no la constante) para su `.w(px(..))` ⇒ `panel_usa_ancho_de_estado`.
- [ ] 5.2 GREEN: elemento `handle-notas` entre `page_area` y `notes_panel` con `on_mouse_down`/`on_mouse_move`/`on_mouse_up`; actualización en vivo + `cx.notify()`; cursor de redimensionado; `cargo test` en verde.

## 6. Persistencia al soltar (RED → GREEN) — `presenter.rs`

- [ ] 6.1 Test (RED): al soltar el asa se invoca `save_notes_width_to` con el ancho saneado (espía/fake de escritura o helper puro `persist_notes_width`) ⇒ `soltar_persiste`.
- [ ] 6.2 GREEN: persistir el ancho en `on_mouse_up`; `cargo test` en verde.

## 7. Cierre

- [ ] 7.1 Auditoría `@react-reviewer`/`@rust-reviewer` (según proceda): sin hallazgos.
- [ ] 7.2 `cargo fmt --all -- --check` y `cargo clippy --all-targets -- -D warnings` limpios; `cargo test` en verde.
- [ ] 7.3 Verificación manual: arrastrar el asa, comprobar límites y que el ancho persiste tras reiniciar.
- [ ] 7.4 `docs/notas-redimensionable.md` creada; archivar el change al mergear.
