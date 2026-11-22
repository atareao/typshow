# Tasks

## 1. Parser del bloque embebido y anclaje (RED → GREEN)

- [x] 1.1 Test (RED): `parse_embedded_blocks(typ)` en `notes_db.rs` extrae un bloque cerrado y devuelve su Markdown (líneas `//` sin prefijo, conservando vacías) ⇒ `parse_embedded_block_basico`.
- [x] 1.2 Test (RED): bloque sin cerrar se descarta y devuelve vacío ⇒ `parse_embedded_block_sin_cerrar`.
- [x] 1.3 Test (RED): varios bloques anclados a sus `==`; bloque sin heading previo se descarta ⇒ `parse_embedded_blocks_anclaje` y `parse_embedded_blocks_sin_heading`.
- [x] 1.4 GREEN: implementar el escáner de líneas con centinelas y la asociación al último heading visto; `cargo test` en verde.

## 2. Sembrado desde bloques y fix del leak (RED → GREEN)

- [x] 2.1 Test (RED): con bloques presentes, `migrate_from_typ` siembra cada página con el Markdown de su bloque ⇒ `migrate_from_blocks`.
- [x] 2.2 Test (RED): sin bloques, el sembrado conserva el comportamiento actual (contenido de slide) ⇒ `migrate_sin_bloques_como_antes`.
- [x] 2.3 Test (RED): ninguna línea `//` del bloque aparece en `content` de segmento ni de nota ⇒ `sin_fuga_de_comentarios`.
- [x] 2.4 GREEN: integrar el sembrado por bloques con *fallback*; excluir el rango de cada bloque del contenido de segmento; `cargo test` en verde.

## 3. Esquema: source_hash y edited (RED → GREEN)

- [x] 3.1 Test (RED): migración idempotente del esquema añade `source_hash TEXT` y `edited INTEGER NOT NULL DEFAULT 0` sin perder filas ⇒ `migracion_esquema_hash`.
- [x] 3.2 GREEN: ampliar `init_schema`/migración y las consultas de `save`/`load` para persistir y leer `source_hash` y `edited`; `cargo test` en verde.

## 4. Reimport selectivo por hash (RED → GREEN)

- [x] 4.1 Test (RED): página no editada con bloque cambiado se refresca y actualiza `source_hash` ⇒ `reimport_refresca_no_editada`.
- [x] 4.2 Test (RED): página editada con bloque cambiado se conserva y se registra divergencia ⇒ `reimport_respeta_editada`.
- [x] 4.3 Test (RED): sin cambios de hash no se toca ninguna fila ⇒ `reimport_sin_cambios`.
- [x] 4.4 Test (RED): la reimportación no borra filas (no `clear_all`) ⇒ `reimport_no_destructiva`.
- [x] 4.5 GREEN: sustituir el `clear_all` de `Notes::reimport` por la lógica selectiva por hash; marcar `edited` al guardar una nota; `cargo test` en verde.

## 5. Export de la BD al .typ (RED → GREEN)

- [x] 5.1 Test (RED): exportar nota nueva inserta bloque tras su heading ⇒ `export_inserta_bloque`.
- [x] 5.2 Test (RED): exportar nota editada reescribe el bloque y deja el resto intacto ⇒ `export_actualiza_bloque`.
- [x] 5.3 Test (RED): nota vacía elimina el bloque ⇒ `export_elimina_bloque_vacio`.
- [x] 5.4 GREEN: implementar `export_to_typ(path, records)` preservando el resto del fichero; cablear una acción en el presentador; `cargo test` en verde.

## 6. Conversor guion.md → bloques (RED → GREEN)

- [x] 6.1 Test (RED): N secciones `## …` se escriben como N bloques tras los N headings de slide, en orden ⇒ `conversor_por_orden`.
- [x] 6.2 Test (RED): reejecutar no duplica bloques ⇒ `conversor_idempotente`.
- [x] 6.3 GREEN: subcomando en `main.rs` (`typshow guion-embed <guion.md> <presentacion.typ>`); `cargo test` en verde.

## 7. Verificación y cierre (REFACTOR)

- [x] 7.1 `cargo fmt --all -- --check` y clippy (`--all-targets -D warnings`) limpios.
- [x] 7.2 `cargo test` en verde, sin regresiones en `notes`/`notes_db`.
- [ ] 7.3 Verificación manual: guion embebido invisible en PDF; notas visibles en el presentador; ciclo sembrado → edición → export → reimport sobre el episodio real.
- [x] 7.4 Crear `docs/guion-embebido.md` y archivar el change (`openspec archive`) al mergear el issue #52. *(doc creada; el archivado se hace al mergear)*

## 8. Remediación de la auditoría (`@rust-reviewer`)

- [x] 8.1 (Bloqueante) Anclaje bloque→página **posicional** (`EmbeddedBlock::heading_line`; `block_for_heading_line` en lugar de `take_block_for`); tests `bloques_con_headings_repetidos_va_a_su_pagina`, `bloques_con_headings_repetidos_en_orden`.
- [x] 8.2 Bloque desaparecido del `.typ`: la nota no editada se vacía y `source_hash = NULL`; si está editada se conserva con `warn!` ⇒ `reimport_limpia_bloque_eliminado`, `reimport_conserva_bloque_eliminado_si_editada`.
- [x] 8.3 `embed_script_into_typ` no cuenta las secciones de cuerpo vacío ⇒ `conversor_no_cuenta_secciones_vacias`.
- [x] 8.4 Documentadas las limitaciones conocidas (BD anterior sin `edited`; heading protegido solo si se edita a mano) + mitigación del heading con `heading_edited` ⇒ `reimport_respeta_heading_editado`, `reimport_actualiza_heading_no_editado`.
- [x] 8.5 El export preserva los finales de línea originales ⇒ `export_preserva_crlf`.
- [x] 8.6 Escape/desescape de centinelas en el contenido ⇒ `round_trip_centinelas_escapados`.
- [x] 8.7 `clear_all` eliminado del código de producción (y su test).
- [x] 8.8 `lib.rs` reexporta `EmbeddedBlock`; `content_hash`, `script_section_bodies`, `embed_script_into_typ` y `render_exported_typ` pasan a `pub(crate)`.
- [x] 8.9 `deck.rs::reimport_notes` guarda la edición en curso antes de reimportar.
- [x] 8.10 Bloque anidado: se descarta el exterior ⇒ `bloque_anidado_descarta_el_exterior`.
- [x] 8.11 `content_hash` pasa a FNV-1a 64 propio (estable, persistido) y se corrige la redacción ⇒ `content_hash_es_fnv1a_estable`.
- [x] 8.12 `PLAN-010.md` §10 y `tasks.md` 7.4 marcados.
- [x] 8.13 Botón «Exportar» con `icons::UPLOAD_SIMPLE` (`\u{E4C0}`) en lugar de `FLOPPY_DISK`, añadido a `icons::ALL`.
