# Handoff — typshow — 2026-11-13 (release 0.2.0 + fix de canales)

## Objetivo
Release 0.2.0 (UI migrada a GPUI) y corrección del bug de color al rasterizar. **Ambas cerradas.**

## Estado
- Hecho: release **0.2.0** publicada (tag `v0.2.0`, `main` en `0.2.0`, `CHANGELOG.md`, GitHub Release con binarios x86_64/aarch64). `development` sincronizado con `main`.
- Hecho: workflow de release corregido (breaking en 0.x → `--minor`; `feat` con gitmoji; `actions: write`; publish a crates.io no bloqueante; `Cargo.lock` sincronizado en cada bump).
- Hecho: **bug de color** arreglado (PR #27, issue #28): `render_page` solo convertía el primer píxel a BGRA → R/B intercambiados (amarillo→cian). Change `fix-canales-color-rasterizado` archivado.
- Hecho: `plans/PLAN-003-ARCHIVED.md`, `docs/release-0.2.0.md`.
- Rama actual: `development` (base `main`)
- Working tree: limpio

## Pendiente
- Verificación visual en Niri (tema claro/oscuro, los 17 iconos, markdown de las notas, `Esc` cierra la proyección) — pendiente; el bug de color se detectó en esta comprobación.
- Editor (follow-ups): undo/redo, navegación por palabras, panel de notas redimensionable.

## Limitaciones conocidas
- crates.io: no publicable mientras `gpui`/`gpui_platform` sean deps git de Zed. Job `publish` en `continue-on-error: true`.
- El paso "Trigger release workflow" ya lleva `actions: write` en `development`; entra en vigor en el próximo merge a `main` (hasta entonces, `release.yml` se dispara a mano).
- El clippy del CI es **1.100.0** (más nuevo que el local): verificar con `rustup run 1.100.0 cargo clippy --all-targets -- -D warnings`.

## Metodología / reglas
SDD (OpenSpec) + TDD (RED→GREEN→REFACTOR). Gitflow: ramas desde `development`, todo por PR, commits `<emoji> <tipo>(<scope>): mensaje`. Delegar la escritura en subagentes.
