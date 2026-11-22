# PLAN-012 — Mapeo por rango de guiones de grano grueso

- **Issue:** #59 · **Rama:** `feature/mapeo-rango-guion` (base `development`) · **OpenSpec:** `openspec/changes/mapeo-rango-guion/`

## Objetivo

Que los guiones de grano grueso (bloques por tramos de tiempo, sin `##`) viajen al `.typ`
como guion embebido: cada bloque se mapea a una sección `=` del `.typ` y su texto se
replica como nota en cada página de esa sección (rango).

## Diseño

- Parser `script_timeline_blocks(md)`: trocea por separadores `─` + cabecera de marca de
  tiempo (`H:MM – H:MM  <emoji> TÍTULO`); ignora preámbulo y cola.
- `section_page_ranges(typ)`: por cada sección `=`, su rango de páginas (hasta la víspera
  de la siguiente `=`).
- `map_blocks_to_sections(b, s)` (pura): identidad si b == s; reparto proporcional monótono
  si difieren.
- `embed_script_ranges_into_typ(script, typ)`: escribe el bloque de cada sección en todas
  las páginas de su rango; idempotente.
- Fallback automático en `guion_embed_files`: `##` ⇒ modo previo; tramos ⇒ modo rango.

## Tareas (TDD, espejo de tasks.md)

- [ ] 1. Parser de bloques por tramos + tests.
- [ ] 2. Rangos de página por sección `=` + tests.
- [ ] 3. Mapeo ordinal/proporcional (puro) + tests.
- [ ] 4. Escritura replicada en el rango + tests (idempotencia).
- [ ] 5. Fallback y CLI + tests.
- [ ] 6. No-render en PDF + no-regresión.
- [ ] 7. Auditoría, fmt/clippy/test, verificación sobre escaleta real, doc y archivado.

## Verificación

```bash
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Estado

- [ ] Pendiente de aprobación del spec por el usuario (fase SDD).
