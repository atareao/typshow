# Handoff — typshow — 2026-11-22

## Objetivo
Botón «Importar guion» **entregado y verificado** (usuario). Sin tareas abiertas.

## Estado (todo mergeado en `development`)
- #52 Guion embebido 1:1 por `##` — PR #53/54.
- #55 Panel de notas redimensionable — PR #56/57.
- #59 Mapeo por rango de guiones de grano grueso — PR #60/61. Doc `docs/mapeo-rango-guion.md`, `PLAN-012-ARCHIVED`.
- #63 Fix: `##` dentro de bloques de código (secuestraba el modo rango) — PR #64/65. Doc `docs/fix-secciones-codigo.md`, `PLAN-013-ARCHIVED`.
- **#67 Botón «Importar guion» en el panel de notas** — PR #68 (`68f0442`), archive #69. Doc `docs/boton-importar-guion.md`, `PLAN-014-ARCHIVED`.
- **Housekeeping:** issues #33/#34/#35/#42/#46/#63 verificados y cerrados. `gh issue list --state open` ⇒ **0**.
- `development` al día. `cargo test` **269 lib + 4 integración**; `clippy -D warnings` y `fmt` limpios.
- **Binario instalado** (`~/.cargo/bin/typshow`, reinstalado con `cargo install --path .`) con #59, #63 y #67.

## Pendiente
- (ninguno conocido). La verificación manual de #67 (tarea 3.3) fue **confirmada por el usuario**.

## Punto de partida
```bash
git checkout development && git pull origin development
# el issue es el punto de entrada:
#   gh issue create ...  ->  openspec new change <feature>  ->  git checkout -b feature/<nombre> development
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Metodología / reglas
SDD (OpenSpec) + TDD: issue → `openspec/changes/<feature>/` → aprobar spec → RED/GREEN/REFACTOR → `openspec archive` al mergear. Rama `feature/*`/`fix/*` desde `development`; PR a `development`; conventional + gitmoji. Nunca editar en `main`/`development` (archivar por rama `chore/` + PR). Plan en `plans/PLAN-<NNN>.md` (siguiente libre: **015**). Doc en `docs/<feature>.md`.

## Riesgos y notas
- **La verificación real/guiada caza lo que los tests no ven:** en #59 se perdió el cuerpo del bloque; en el opcional, `script_section_bodies` contaba `##` en bloques de código. Mantener verificación sobre datos reales.
- **`@backend-dev`** es el subagente fiable para implementar; `@rust-writer` falló con no-ops. **Verificar con CLI** siempre (los resúmenes a veces omiten las salidas literales).
- **Panel de notas / importar guion — limitaciones** (`docs/boton-importar-guion.md`): requiere un `.typ` abierto; estado parcial si falla la reimportación tras escribir; el feedback cuenta bloques generados (no cambios).
- Config: rama de `~/.config/opencode` sin push (gitea); `~/.agents` es repo local.
- Backup de `presentaciones/`: `/tmp/opencode/backup-presentaciones/presentaciones-20261122-102440.tgz`.
