# Tasks

## 1. Detección de secciones consciente de fences (RED → GREEN) — `notes_db.rs`

- [x] 1.1 Test (RED): `## ` dentro de un bloque cercado (```) NO cuenta como sección ⇒ `seccion_dentro_de_fence_ignorada`.
- [x] 1.2 Test (RED): `    ## ` (4 espacios) fuera de fence NO cuenta como sección ⇒ `cabecera_indentada_ignorada`.
- [x] 1.3 Test (RED): `## ` reales siguen contando (no regresión) ⇒ `secciones_reales_intactas`.
- [x] 1.4 GREEN: implementar el escáner de fences + regla de indentación ≤ 3; `cargo test` verde.

## 2. La escaleta 807 vuelve al modo por rango (RED → GREEN)

- [x] 2.1 Test (RED): un guion con tramos `H:MM – H:MM` **y** `##` embebido en código ⇒ `script_section_bodies` vacío y `embed_script_into_typ` usa el modo rango ⇒ `tramos_con_markdown_embebido_usa_rango`.
- [x] 2.2 GREEN: verificado el fallback; `cargo test` verde.

## 3. Cierre

- [x] 3.1 Auditoría `@rust-reviewer`: APROBADA (3 nits menores documentados).
- [x] 3.2 `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- [x] 3.3 Re-ejecutado sobre la 807: 7 bloques con cuerpo; 15/15 escaletas correctas.
- [ ] 3.4 Doc (`docs/`) y archivar el change al mergear.
