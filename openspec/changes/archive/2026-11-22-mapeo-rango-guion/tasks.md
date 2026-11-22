# Tasks

## 0. Preparación

- [x] 0.1 Rama `feature/mapeo-rango-guion` desde `development`; issue #59 referenciado.

## 1. Parser de bloques por tramos de tiempo (RED → GREEN) — `notes_db.rs`

- [x] 1.1 Test (RED): `script_timeline_blocks(md)` extrae los bloques entre separadores `─` con cabecera de marca de tiempo, en orden ⇒ `bloques_por_tramos_basico`.
- [x] 1.2 Test (RED): ignora el preámbulo anterior al primer bloque (título del episodio) y el material posterior al último ⇒ `bloques_ignoran_preambulo_y_cola`.
- [x] 1.3 Test (RED): un guion sin separadores de tramo devuelve vacío ⇒ `sin_tramos_devuelve_vacio`.
- [x] 1.4 GREEN: implementar el escáner de separadores + cabecera; `cargo test` en verde.

## 2. Rangos de página por sección `=` (RED → GREEN) — `notes_db.rs`

- [x] 2.1 Test (RED): `section_page_ranges(typ)` devuelve, por cada sección `=`, el rango de páginas que cubre (su página a la anterior a la siguiente `=`) ⇒ `rangos_por_seccion`.
- [x] 2.2 Test (RED): la portada (`title:`) y una `==` antes de la primera `=` no forman sección ⇒ `rangos_ignoran_portada`.
- [x] 2.3 GREEN: implementar sobre la línea de cada heading; `cargo test` en verde.

## 3. Mapeo ordinal/proporcional (RED → GREEN) — `notes_db.rs`

- [x] 3.1 Test (RED): `map_blocks_to_sections(b, s)` con b == s da la identidad ⇒ `mapeo_identidad`.
- [x] 3.2 Test (RED): con b < s reparte monótono cubriendo todas las secciones (una sección recibe el bloque más cercano) ⇒ `mapeo_b_menor_s`.
- [x] 3.3 Test (RED): con b > s los bloques sobrantes se agregan a la misma sección ⇒ `mapeo_b_mayor_s`.
- [x] 3.4 Test (RED): b == 0 o s == 0 devuelve vacío sin pánico ⇒ `mapeo_vacio`.
- [x] 3.5 GREEN: implementar el reparto; `cargo test` en verde.

## 4. Escritura del bloque en cada página del rango (RED → GREEN) — `notes_db.rs`

- [x] 4.1 Test (RED): `embed_script_ranges_into_typ(script, typ)` escribe el bloque de cada sección en TODAS las páginas de su rango (replicado) ⇒ `embed_replica_en_rango`.
- [x] 4.2 Test (RED): es idempotente (reejecutar no duplica bloques) ⇒ `embed_rango_idempotente`.
- [x] 4.3 GREEN: reutilizar `parse_embedded_blocks`/`render_exported_typ` para insertar en cada heading del rango; `cargo test` en verde.

## 5. Fallback y CLI (RED → GREEN) — `notes_db.rs`, `main.rs`

- [x] 5.1 Test (RED): `guion_embed_files` con un guion de grano grueso (sin `##`) usa el mapeo por rango ⇒ `cli_fallback_tramos`.
- [x] 5.2 Test (RED): con secciones `##` se usa el comportamiento previo ⇒ `cli_usa_secciones_cuando_existen`.
- [x] 5.3 Test (RED): guion sin `##` ni tramos ⇒ 0 bloques + `warn!`, sin abortar ⇒ `cli_sin_bloques_avisa`.
- [x] 5.4 GREEN: encadenar detección → mapeo por rango en `guion_embed_files`; `cargo test` en verde.

## 6. No-render y no-regresión (RED → GREEN)

- [x] 6.1 Test: el guion embebido por rango no aparece en el PDF ⇒ `guion_rango_no_aparece_en_el_pdf` (`tests/pipeline_test.rs`).
- [x] 6.2 GREEN: verificar la ruta de exclusión de bloques (`embedded_block_ranges`); `cargo test` en verde.

## 7. Cierre

- [x] 7.1 Auditoría `@rust-reviewer`: APROBADA (1ª ronda: 1 mayor + 3 menores + bug de cuerpo cazado en 7.3; remediados y re-auditados).
- [x] 7.2 `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test` limpios.
- [x] 7.3 Verificación real sobre `episodio-812-escaleta-wave-terminal.md` + su `.typ`: 7 bloques embebidos, cuerpo visible, idempotente.
- [x] 7.4 `docs/mapeo-rango-guion.md` creada; change archivado al mergear (PR #60).
