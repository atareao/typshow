# Tasks

## 0. Preparación

- [ ] 0.1 Rama `feature/mapeo-rango-guion` desde `development`; issue #59 referenciado.

## 1. Parser de bloques por tramos de tiempo (RED → GREEN) — `notes_db.rs`

- [ ] 1.1 Test (RED): `script_timeline_blocks(md)` extrae los bloques entre separadores `─` con cabecera de marca de tiempo, en orden ⇒ `bloques_por_tramos_basico`.
- [ ] 1.2 Test (RED): ignora el preámbulo anterior al primer bloque (título del episodio) y el material posterior al último ⇒ `bloques_ignoran_preambulo_y_cola`.
- [ ] 1.3 Test (RED): un guion sin separadores de tramo devuelve vacío ⇒ `sin_tramos_devuelve_vacio`.
- [ ] 1.4 GREEN: implementar el escáner de separadores + cabecera; `cargo test` en verde.

## 2. Rangos de página por sección `=` (RED → GREEN) — `notes_db.rs`

- [ ] 2.1 Test (RED): `section_page_ranges(typ)` devuelve, por cada sección `=`, el rango de páginas que cubre (su página a la anterior a la siguiente `=`) ⇒ `rangos_por_seccion`.
- [ ] 2.2 Test (RED): la portada (`title:`) y una `==` antes de la primera `=` no forman sección ⇒ `rangos_ignoran_portada`.
- [ ] 2.3 GREEN: implementar sobre la línea de cada heading; `cargo test` en verde.

## 3. Mapeo ordinal/proporcional (RED → GREEN) — `notes_db.rs`

- [ ] 3.1 Test (RED): `map_blocks_to_sections(b, s)` con b == s da la identidad ⇒ `mapeo_identidad`.
- [ ] 3.2 Test (RED): con b < s reparte monótono cubriendo todas las secciones (una sección recibe el bloque más cercano) ⇒ `mapeo_b_menor_s`.
- [ ] 3.3 Test (RED): con b > s los bloques sobrantes se agregan a la misma sección ⇒ `mapeo_b_mayor_s`.
- [ ] 3.4 Test (RED): b == 0 o s == 0 devuelve vacío sin pánico ⇒ `mapeo_vacio`.
- [ ] 3.5 GREEN: implementar el reparto; `cargo test` en verde.

## 4. Escritura del bloque en cada página del rango (RED → GREEN) — `notes_db.rs`

- [ ] 4.1 Test (RED): `embed_script_ranges_into_typ(script, typ)` escribe el bloque de cada sección en TODAS las páginas de su rango (replicado) ⇒ `embed_replica_en_rango`.
- [ ] 4.2 Test (RED): es idempotente (reejecutar no duplica bloques) ⇒ `embed_rango_idempotente`.
- [ ] 4.3 GREEN: reutilizar `parse_embedded_blocks`/`render_exported_typ` para insertar en cada heading del rango; `cargo test` en verde.

## 5. Fallback y CLI (RED → GREEN) — `notes_db.rs`, `main.rs`

- [ ] 5.1 Test (RED): `guion_embed_files` con un guion de grano grueso (sin `##`) usa el mapeo por rango ⇒ `cli_fallback_tramos`.
- [ ] 5.2 Test (RED): con secciones `##` se usa el comportamiento previo ⇒ `cli_usa_secciones_cuando_existen`.
- [ ] 5.3 Test (RED): guion sin `##` ni tramos ⇒ 0 bloques + `warn!`, sin abortar ⇒ `cli_sin_bloques_avisa`.
- [ ] 5.4 GREEN: encadenar detección → mapeo por rango en `guion_embed_files`; `cargo test` en verde.

## 6. No-render y no-regresión (RED → GREEN)

- [ ] 6.1 Test: el guion embebido por rango no aparece en el PDF ⇒ `guion_rango_no_aparece_en_el_pdf` (`tests/pipeline_test.rs`).
- [ ] 6.2 GREEN: verificar la ruta de exclusión de bloques (`embedded_block_ranges`); `cargo test` en verde.

## 7. Cierre

- [ ] 7.1 Auditoría `@rust-reviewer`: sin hallazgos.
- [ ] 7.2 `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test` limpios.
- [ ] 7.3 Verificación real sobre `episodio-812-escaleta-wave-terminal.md` + su `.typ`: guion embebido y visible en el presentador.
- [ ] 7.4 `docs/mapeo-rango-guion.md` y archivar el change al mergear.
