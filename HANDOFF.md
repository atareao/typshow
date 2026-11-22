# Handoff — typshow — 2026-11-22

## Objetivo
Ciclo de guion embebido **cerrado** (mapeo por rango + fix de fences) y **housekeeping hecho**.
Sin tareas pendientes conocidas; abrir la siguiente feature desde un issue.

## Estado (todo mergeado en `development`)
- #52 Guion embebido 1:1 por secciones `##` — PR #53, archive #54.
- #55 Panel de notas redimensionable — PR #56, archive #57.
- #59 Mapeo por rango de guiones de grano grueso — PR #60 (`43402c0`), archive #61 (`83d364e`). Doc `docs/mapeo-rango-guion.md`, plan `PLAN-012-ARCHIVED`.
- **#63 Fix: `##` dentro de bloques de código** (secuestraba el modo rango) — PR #64 (`58a8961`), archive #65 (`e3e879a`). Doc `docs/fix-secciones-codigo.md`, plan `PLAN-013-ARCHIVED`.
- **Housekeeping:** issues #33, #34, #35, #42, #46 verificados (código + change archivado + PR) y **cerrados**. `gh issue list --state open` ⇒ **0 abiertos**.
- **Datos (fuera del repo):** las **15** presentaciones de `presentaciones/` (episodios 805–818, 822) tienen ya el guion embebido por rango (7–8 bloques, con cuerpo, idempotente). Backup previo en `/tmp/opencode/backup-presentaciones/presentaciones-20261122-102440.tgz` (446 M).
- `development` al día. Árbol limpio. `cargo test` **264 lib + 4 integración**; `clippy -D warnings` y `fmt` limpios.

## Pendiente
- (ninguno conocido). Ideas: re-ejecutar `guion-embed` sobre episodios futuros en el pipeline (el modo por rango es automático).

## Punto de partida
```bash
git checkout development && git pull origin development
# el issue es el punto de entrada:
#   gh issue create ...  ->  openspec new change <feature>  ->  git checkout -b feature/<nombre> development
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Metodología / reglas
SDD (OpenSpec) + TDD: issue como punto de entrada → `openspec/changes/<feature>/` → aprobar spec → RED/GREEN/REFACTOR → `openspec archive` al mergear. Rama `feature/*`/`fix/*` desde `development`; PR a `development`; conventional + gitmoji. Nunca editar en `main`/`development` (archivados por rama `chore/` + PR). Plan en `plans/PLAN-<NNN>.md` (siguiente libre: **014**). Doc de feature en `docs/<feature>.md`.

## Riesgos y notas
- **La verificación real caza lo que la auditoría estática no ve:** en #59 se descubrió que se perdía el **cuerpo** del bloque; en el opcional, que `script_section_bodies` contaba `##` dentro de bloques de código. Mantener SIEMPRE la verificación sobre datos reales.
- **Subagentes:** `@backend-dev` es el fiable para implementar; `@rust-writer` falló con no-ops fabricados. **Verificar con CLI** (`git diff --stat`, `rg` de símbolos, `cargo test`) antes de aceptar; los resúmenes no incluyen siempre las salidas literales.
- **Mapeo por rango — limitaciones** (`docs/mapeo-rango-guion.md`): la cola sin marca horaria (p. ej. `CHECKLIST`) se anexa a la última sección; una hora en una línea de cuerpo puede truncar el bloque.
- **Detección `##`** (`docs/fix-secciones-codigo.md`): tab de indentación cuenta como 1 espacio; acepta `##` desnudo.
- Config: rama de `~/.config/opencode` **sin push** (gitea); `~/.agents` es repo local.
- `.typshow.db` sidecar del episodio `el-dia…` ya sembrado.
