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

- [x] 1. Parser de bloques por tramos + tests.
- [x] 2. Rangos de página por sección `=` + tests.
- [x] 3. Mapeo ordinal/proporcional (puro) + tests.
- [x] 4. Escritura replicada en el rango + tests (idempotencia).
- [x] 5. Fallback y CLI + tests.
- [x] 6. No-render en PDF + no-regresión.
- [x] 7. Auditoría APROBADA, fmt/clippy/test limpios y verificación real sobre la 812 (el fix del cuerpo se halló en 7.3). Doc pendiente de archivar al mergear.

## Verificación

```bash
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Estado

- [x] Spec aprobado por el usuario; implementado en TDD (RED→GREEN→REFACTOR) en `feature/mapeo-rango-guion`.
