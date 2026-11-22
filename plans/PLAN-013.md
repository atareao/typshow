# PLAN-013 — Detección de `##` fuera de bloques de código

- **Issue:** #63 · **Rama:** `fix/secciones-dentro-de-codigo` (base `development`) · **OpenSpec:** `openspec/changes/fix-secciones-codigo/`

## Objetivo

Que `script_section_bodies` no confunda el Markdown embebido en bloques de código con
secciones `##`, de modo que las escaletas con fences vuelvan al modo por rango (#59).

## Diseño

- Helper de fences: recorrer el guion marcando si cada línea cae dentro de ``` ``` ```.
- Una línea es cabecera si: está fuera de fence, tiene ≤ 3 espacios de indentación y empieza
  por `## ` (o exactamente `##`).

## Tareas (TDD, espejo de tasks.md)

- [x] 1. Fences + indentación en `script_section_bodies` + tests.
- [x] 2. Regresión del fallback (tramos con Markdown embebido ⇒ modo rango).
- [x] 3. Auditoría APROBADA, fmt/clippy/test limpios, re-ejecución sobre la 807 (7 bloques). Doc pendiente de archivar al mergear.

## Verificación

```bash
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Estado

- Spec aprobado; tareas 1–2 en verde (TDD RED→GREEN). Pendientes: auditoría, re-ejecución sobre la 807, doc y archivado.
