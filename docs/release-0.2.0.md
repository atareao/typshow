# Release 0.2.0

Primera release de Typshow con la UI migrada de `egui`/`eframe` a **GPUI** (Zed). Bump `minor` en la serie `0.x`.

## Resultado
- Tag `v0.2.0` y `main` en `0.2.0`; `CHANGELOG.md` regenerado con git-cliff.
- **GitHub Release `v0.2.0`** con binarios Linux `x86_64` y `aarch64`.
- `development` sincronizado con `main`.
- **crates.io: NO publicado** (limitación, ver abajo).

## Ficheros implicados
- `.github/workflows/release-prepare.yml` — cálculo del bump y preparación de la release en `main`.
- `.github/workflows/release.yml` — build multi-plataforma, GitHub Release y publicación.
- `.vampus.yml` y `Cargo.toml` — versión.
- `plans/PLAN-003-ARCHIVED.md` — plan de la release.

## Cómo funciona el bump
`release-prepare.yml` decide el tipo de incremento a partir de los commits desde el último tag:
- `BREAKING CHANGE` o `!:` → `--major`, **salvo** en la serie `0.x`, donde semver exige `--minor`.
- `feat` (con o sin gitmoji) → `--minor`.
- resto → `--patch`.

Después `vampus upgrade <tipo>` actualiza `.vampus.yml`/`Cargo.toml`, se genera el CHANGELOG, se crea el tag y se dispara `release.yml`.

## Limitaciones conocidas
- **crates.io no es publicable** mientras `gpui`/`gpui_platform` sean dependencias `git` de Zed (no existen en crates.io). El job `publish` queda con `continue-on-error: true`; el canal real son los binarios de GitHub Releases.
- El paso "Trigger release workflow" de `release-prepare.yml` necesita `actions: write` en `permissions`; sin él, `gh workflow run` responde 403 y hay que lanzar `release.yml` a mano.

## Configuración
- Secreto `CARGO_REGISTRY_TOKEN` (sin efecto mientras crates.io esté bloqueado).
- `.vampus.yml`: `current_version` y `replaces` (patrón sobre `Cargo.toml`).

## Follow-ups
- Reintentar crates.io si Zed publica `gpui`/`gpui_platform`, o valorar un fork publicado.
- Verificación visual en Niri (tema claro/oscuro, los 17 iconos, markdown de las notas, `Esc` cierra la proyección).
