# PLAN-003 — Publicar release 0.2.0 (Typshow) [ARCHIVADO]

- **Issue:** #21
- **Rama:** `chore/release-0.2.0` (desde `development`)
- **Estado:** completado (2026-11-13)

## Objetivo
Publicar la release **0.2.0** (minor) tras la migración de la UI de egui a GPUI.

## Resultado
- Tag `v0.2.0`, `main` en `0.2.0`, CHANGELOG y **GitHub Release `v0.2.0`** (binarios x86_64 + aarch64).
- `development` sincronizado con `main`.
- crates.io **no publicado**: `gpui`/`gpui_platform` son deps git de Zed y no están en crates.io → job `publish` marcado `continue-on-error: true`.

## Bugs corregidos en `.github/workflows/release-prepare.yml`
1. Breaking en `0.x` ahora es `--minor` (antes `--major` → habría publicado `1.0.0`).
2. Detección de `feat` con gitmoji (`grep -qE '(^| )(feat|✨)'`).
3. `permissions: actions: write` para que `gh workflow run release.yml` no dé 403.

## Tareas
- [x] T1. Rama `chore/release-0.2.0` e issue #21.
- [x] T2. Corregir el paso de bump (2 bugs).
- [x] T3. Simulación local contra `v0.1.11..development` ⇒ `--minor`.
- [x] T4. `vampus preview --minor` ⇒ `0.2.0`.
- [x] T5. Verificación (CI en verde en PRs #22 y #23).
- [x] T6. Commit + push + PR #22 a `development` + merge.
- [x] T7. PR #23 `development` → `main` + merge (disparó la release).
- [x] T8. Verificado GitHub Release `v0.2.0`; crates.io bloqueado (no bloqueante).
- [x] T9. Ramas mergeadas borradas.
- [x] T10. `docs/release-0.2.0.md` y este plan archivado.

## Riesgos
- Merge a `main` publica de forma irreversible (mitigado con `--minor` verificado antes).
