# Handoff — typshow — 2026-11-22

## Objetivo
Nada pendiente. **v0.6.0 publicada** (línea de guion embebido). Repo limpio: 0 issues, 0 TODOs, solo `main`/`development`.

## Estado (todo mergeado y liberado)
- **v0.6.0** (GitHub Release + tag `v0.6.0`, assets linux x86_64/aarch64). PR #73, release commit por el bot. `main` ↔ `development` sincronizadas.
  - #52 Guion embebido 1:1 por `##` · #55 Panel de notas redimensionable · #59 Mapeo por rango · #63 Fix `##` en bloques de código · #67 Botón «Importar guion».
- Housekeeping: issues #33/#34/#35/#42/#46/#63/#67 cerrados; planes 005–014 archivados; ramas obsoletas borradas (solo quedan `main`/`development`).
- Docs: `docs/mapeo-rango-guion.md`, `docs/fix-secciones-codigo.md`, `docs/boton-importar-guion.md`.
- `cargo test` **269 lib + 4 integración**; `clippy -D warnings` y `fmt` limpios.
- **Binario instalado** `~/.cargo/bin/typshow` = **v0.6.0**.
- Datos (fuera del repo): 15 presentaciones (805–818, 822) con guion embebido por rango. Backup: `/tmp/opencode/backup-presentaciones/presentaciones-20261122-102440.tgz`.

## Pendiente
- (ninguno conocido).

## Punto de partida
```bash
git checkout development && git pull origin development
# el issue es el punto de entrada:
#   gh issue create ...  ->  openspec new change <feature>  ->  git checkout -b feature/<nombre> development
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Metodología / reglas
SDD (OpenSpec) + TDD: issue → `openspec/changes/<feature>/` → aprobar spec → RED/GREEN/REFACTOR → `openspec archive` al mergear. Rama `feature/*`/`fix/*` desde `development`; PR a `development`; conventional + gitmoji. **Release**: PR `development`→`main` y merge → `release-prepare.yml` (bump+changelog+tag+sync) + `release.yml` (build+GitHub Release). Plan en `plans/PLAN-<NNN>.md` (**siguiente libre: 015**). Doc en `docs/<feature>.md`.

## Riesgos y notas
- **Verificación real sobre datos**, no solo auditoría estática: en #59 se perdió el cuerpo del bloque; en el opcional, `script_section_bodies` contaba `##` dentro de código.
- **`@backend-dev`** es el implementador fiable; `@rust-writer` falló con no-ops. Verificar con CLI siempre.
- **crates.io**: `cargo publish` es `continue-on-error: true` **a propósito** (gpui/gpui_platform son deps git). El canal real es el GitHub Release.
- Limitaciones conocidas en `docs/mapeo-rango-guion.md` y `docs/boton-importar-guion.md`.
- Config: rama de `~/.config/opencode` sin push (gitea); `~/.agents` repo local.
