# PLAN-010 — Guion embebido en el .typ como notas del presentador

- **Issue:** #52
- **Rama:** `feature/guion-embebido-en-typ` (desde `development`)
- **OpenSpec:** `openspec/changes/guion-embebido-en-typ/`
- **Estado:** implementado (TDD) en `feature/guion-embebido-en-typ`; pendiente auditoría, verificación manual en GUI y cierre del change

## Objetivo

Que el guion del episodio viaje embebido dentro del `.typ` de la presentación,
invisible en el PDF (comentarios de Typst) y visible solo en typshow como notas del
presentador, con reimportación selectiva y export bidireccional.

## Alcance

- Formato: bloque delimitado por centinelas en comentarios `// >>> typshow:notes` …
  `// >>> typshow:notes:end`, anclado al heading `=`/`==` inmediatamente anterior.
- Sembrado de notas desde el bloque (con *fallback* al comportamiento actual si no
  hay bloques) y fix del *leak*: las líneas `//` no deben colarse como contenido.
- Esquema de `notes` con `source_hash` y `edited`; reimport selectivo por hash,
  nunca destructivo.
- Export de la BD a los bloques del `.typ`, preservando el resto del fichero.
- Subcomando `typshow guion-embed <guion.md> <presentacion.typ>` idempotente.

## Tareas

- [x] 1. Parser del bloque embebido y anclaje a página (RED → GREEN) — `notes_db.rs`.
- [x] 2. Sembrado desde bloques + fix del *leak* de comentarios (RED → GREEN).
- [x] 3. Esquema: `source_hash` y `edited` con migración idempotente (RED → GREEN).
- [x] 4. Reimport selectivo por hash, no destructivo (RED → GREEN) — `notes.rs`.
- [x] 5. Export de la BD al `.typ`, preservando el resto (RED → GREEN).
- [x] 6. Conversor `guion.md → bloques` como subcomando de CLI (RED → GREEN) — `main.rs`.
- [ ] 7. Auditoría `@rust-reviewer`: aprobado. *(1ª ronda: cambios requeridos; los 13 puntos están remediados con tests — ver §8 de `openspec/changes/guion-embebido-en-typ/tasks.md`. Pendiente la re-auditoría.)*
- [x] 8. `cargo fmt` y clippy (`-D warnings`) limpios; `cargo test` en verde.
- [ ] 9. Verificación manual sobre el episodio real (invisible en PDF, visible en presentador).
- [x] 10. Crear `docs/guion-embebido.md` y archivar el change OpenSpec al mergear. *(doc creada; el `openspec archive` se hace al mergear)*

## Criterios de aceptación

- Un `.typ` con bloques embebidos compila a PDF sin rastro del guion.
- El presentador muestra el guion de cada página como nota.
- Editar una nota y exportar actualiza el bloque del `.typ` sin tocar el resto.
- Cambiar un bloque en el `.typ` refresca la nota solo si no fue editada.
- Sin regresiones: `cargo test` en verde.
