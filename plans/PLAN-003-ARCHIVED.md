# PLAN-003 — Publicar release 0.2.0 (Typshow)

- **Issue:** #21
- **Rama:** `chore/release-0.2.0` (desde `development`)
- **Fecha:** 2026-11-13
- **Estado:** en curso

## Objetivo

Publicar la release **0.2.0** (minor) en crates.io + GitHub tras la migración de la UI de egui a GPUI.

## Contexto / hallazgos

- `development` está 38 commits por delante de `main`; merge-base = `8b2c7f5`.
- Último tag: `v0.1.11` (en `main`, commit `1327066`). `main` lleva `.vampus.yml`/`Cargo.toml` a `0.1.11`; `development` a `0.1.10` (sync pendiente).
- El commit `f97a318` (`💥 feat(gpui)!: ...`) contiene `!:` → el workflow aplicaría `--major` → `1.0.0` (incorrecto). Decisión del usuario: `0.2.0`.

## Bugs detectados en `.github/workflows/release-prepare.yml` (paso "Determine bump type from commits")

1. **Breaking en 0.x**: `!:` fuerza `--major` sin considerar que la major actual es `0`. Semver: breaking en `0.x` ⇒ `--minor`.
2. **Detección de `feat` rota**: `grep -q "^feat"` no casa nunca porque los asuntos empiezan por gitmoji (`✨ feat(...)`). Un `feat` se clasifica como patch.

## Contratos (nuevo bloque de decisión de bump)

- `CURRENT` = `current_version` de `.vampus.yml`; `MAJOR` = primer componente.
- Si `COMMITS` casa `BREAKING CHANGE|!:`:
  - `MAJOR == 0` ⇒ `--minor`
  - si no ⇒ `--major`
- Si no, si `COMMITS` casa `(^| )(feat|✨)` ⇒ `--minor`
- Si no ⇒ `--patch`
- El resultado se escribe en `$GITHUB_OUTPUT` como `type=--...`.

## Escenarios (verificación)

- **Given** `current_version: 0.1.11` y un rango con `💥 feat(gpui)!:` — **When** corre la detección — **Then** `type=--minor` y `vampus preview --minor` ⇒ `0.2.0`.
- **Given** `current_version: 1.2.0` y un rango con `!:` — **When** corre la detección — **Then** `type=--major`.
- **Given** un rango con `✨ feat(...)` y sin breaking — **When** corre la detección — **Then** `type=--minor`.
- **Given** un rango solo con `🐛 fix`, `📝 docs`, `♻️ refactor` — **When** corre la detección — **Then** `type=--patch`.

## Tareas

- [x] T1. Crear rama `chore/release-0.2.0` e issue #21.
- [ ] T2. Corregir el paso de bump en `release-prepare.yml` (2 bugs).
- [ ] T3. Simular la lógica local contra `v0.1.11..development` ⇒ esperado `--minor`.
- [ ] T4. `vampus preview --minor` ⇒ `0.2.0`.
- [ ] T5. Verificación: `cargo test`, `cargo fmt --all -- --check`, `rustup run 1.99.1 cargo clippy --all-targets -- -D warnings`.
- [ ] T6. Commit gitmoji + push + PR a `development` + merge.
- [ ] T7. PR `development` → `main` (dispara CI: bump 0.2.0, CHANGELOG, tag `v0.2.0`, publish).
- [ ] T8. Verificar GitHub Release `v0.2.0` y crates.io.
- [ ] T9. Borrar ramas mergeadas: `feature/gpui-migration`, `feature/gpui-spike`.
- [ ] T10. Crear `docs/release-0.2.0.md` y renombrar a `plans/PLAN-003-ARCHIVED.md`.

## Riesgos

- Merge a `main` publica en crates.io de forma **irreversible**: verificar el bump antes.
- El paso "Sync development with main" del workflow sobrescribe `development`; sin trabajo concurrente ahora.
- Clippy del runner más nuevo que el local: validar con 1.99.1.