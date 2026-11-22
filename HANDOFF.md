# Handoff — typshow — 2026-11-22

## Objetivo
Línea del guion embebido **cerrada**. Siguiente: housekeeping de issues antiguos y/o nueva feature.

## Estado
- **Completado y mergeado en `development`:**
  - #52 Guion embebido 1:1 por secciones `##` — PR #53, archive PR #54.
  - #55 Panel de notas redimensionable — PR #56, archive PR #57.
  - **#59 Mapeo por rango de guiones de grano grueso** — PR #60 (`43402c0`), archive PR #61 (`83d364e`). Doc `docs/mapeo-rango-guion.md`, plan `plans/PLAN-012-ARCHIVED.md`.
- `development` al día (`83d364e`). Árbol limpio. Sin ramas feature/chore colgando.
- Verificación: `cargo test` **259 lib + 4 integración**; `clippy -D warnings` y `fmt` limpios. Auditorías `@rust-reviewer` APROBADAS.
- Verificación real #59 sobre `episodio-812-escaleta-wave-terminal.md` + su `.typ`: 7 bloques embebidos con cuerpo, visible en el presentador, invisible en PDF, idempotente.

## Pendiente
1. **Housekeeping:** issues antiguos abiertos que parecen ya implementados — revisar y cerrar si procede: **#46, #42, #35, #34, #33** (hay docs en `docs/`).
2. **Guion embebido — resto de episodios:** el modo por rango es automático (sin flag), así que el pipeline ya lo aprovecha; opcionalmente re-ejecutar `typshow guion-embed` sobre los episodios de grano grueso que falten.

## Punto de partida
```bash
git checkout development && git pull origin development
# el issue es el punto de entrada:
#   gh issue create ...  ->  openspec new change <feature>  ->  git checkout -b feature/<nombre> development
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Metodología / reglas
SDD (OpenSpec) + TDD: issue como punto de entrada → `openspec/changes/<feature>/` → aprobar spec → RED/GREEN/REFACTOR → `openspec archive` al mergear. Rama `feature/*` desde `development`; PR a `development`; commits conventional + gitmoji. Nunca editar en `main`/`development` (archivados por rama `chore/` + PR). Plan en `plans/PLAN-<NNN>.md` (siguiente libre: **013**). Doc de feature en `docs/<feature>.md`.

## Riesgos y notas
- **Verificación real > auditoría estática:** en #59 la auditoría no cazó que `script_timeline_blocks` perdía el cuerpo del bloque; lo detectó la corrida sobre la 812. Mantener la tarea de verificación real.
- **Subagentes:** `@rust-writer` falló dos veces (no-op fabricado); `@backend-dev` es el que implementa de forma fiable. **Verificar siempre con CLI** (`git diff --stat`, `rg` de símbolos, `cargo test`) antes de aceptar.
- **Mapeo por rango — limitaciones** (ver `docs/mapeo-rango-guion.md`): la cola sin marca horaria (p. ej. `CHECKLIST`) se anexa a la última sección; una hora en una línea de cuerpo puede truncar el bloque.
- Config: rama de `~/.config/opencode` **sin push** (gitea); `~/.agents` es repo local.
- `.typshow.db` sidecar del episodio `el-dia…` ya sembrado; copia con guion embebido en la carpeta del episodio.
