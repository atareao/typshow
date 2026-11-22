# Handoff — typshow — 2026-11-22

## Objetivo
Cerrar la línea de trabajo inmediata (hecho) y arrancar la siguiente: **decidir la estrategia del punto #3** (importar más episodios al guion embebido).

## Estado
- **Completado y mergeado en `development`:**
  - #52 «Guion embebido en el `.typ` como notas del presentador» — PR #53 (`69bc960`), change archivado (PR #54). Issue #52 **cerrado**. Doc `docs/guion-embebido.md`.
  - #55 «Panel de notas redimensionable (asa, ancho mín/máx, persistencia)» — PR #56 (`55e1cbc`), change archivado (PR #57, `d86e321`). Issue #55 **cerrado**. Plan `plans/PLAN-011-ARCHIVED.md`, doc `docs/notas-redimensionable.md`.
- `development` al día (`d86e321`). Árbol limpio. Sin ramas feature/chore colgando.
- Verificación: `cargo test` 241 lib + 3 integración; `clippy -D warnings` y `fmt` limpios. Auditoría `@rust-reviewer` APROBADA.

## Pendiente
1. **Decidir punto #3** (mejorar el embedding para importar más episodios): A (match por título) / B (mapeo por rango, **recomendada**) / C (guion global). Requiere issue + OpenSpec antes de código.
2. **Housekeeping:** quedan issues antiguos abiertos que parecen ya implementados — revisar y cerrar si procede: #46, #42, #35, #34, #33 (hay docs en `docs/`).

## Punto de partida
```bash
git checkout development && git pull origin development
# siguiente feature:
#   gh issue create ...   # el issue es el punto de entrada
#   openspec new change <feature>
#   git checkout -b feature/<nombre> development
# verificación:
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Metodología / reglas
SDD (OpenSpec) + TDD: issue como punto de entrada → `openspec/changes/<feature>/` → aprobar spec → RED/GREEN/REFACTOR → `openspec archive` al mergear. Rama `feature/*` desde `development`; PR a `development`; commits conventional + gitmoji. Nunca editar en `main`/`development` (los archivados van por rama `chore/` + PR). Plan en `plans/PLAN-<NNN>.md` (siguiente libre: **012**). Doc de feature en `docs/<feature>.md`.

## Riesgos y notas
- Los ~26 episodios con guion de grano grueso **no** casan 1:1; solo 8 sí (`de-huggingface…`, `docker-no-es-podman-rootless`, `el-dia…`, `el-self-hosting…`, `github-a-gitea…`, `herramientas-para-valet`, `que-tu-ia-te-dibuje…`, `tu-nube-es-tuya…`). Recomendado prototipo de B sobre un episodio de grano grueso.
- Config: rama de `~/.config/opencode` **sin push** (gitea); `~/.agents` es repo local.
- `.typshow.db` sidecar del episodio `el-dia…` ya sembrado; copia con guion embebido en la carpeta del episodio.
