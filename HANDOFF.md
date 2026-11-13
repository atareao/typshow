# Handoff — typshow — 2026-11-13 (release 0.2.1 publicada)

## Objetivo
Corregir el bug de color al rasterizar y publicarlo. **Publicado como 0.2.1.**

## Estado
- Hecho: **release 0.2.1** publicada (tag `v0.2.1`, `main` en `0.2.1`, `CHANGELOG.md`, GitHub Release con binarios x86_64/aarch64). PRs #31 (release) y #27/#29/#30 mergeados; `development` sincronizado con `main`.
- Hecho: release 0.2.0 previa publicada (tag `v0.2.0`).
- Hecho: **bug de color** arreglado (PR #27, issue #28): `render_page` solo convertía el primer píxel a BGRA → R/B intercambiados (amarillo→cian). Change `fix-canales-color-rasterizado` archivado.
- Hecho: **verificación visual en Niri** del fix: el logo (Tux) muestra el pico/patas amarillos correctos.
- Hecho: workflow de release corregido (breaking en 0.x → `--minor`; `feat` con gitmoji; `actions: write`; publish a crates.io no bloqueante; `Cargo.lock` sincronizado).
- Hecho: `plans/PLAN-004-ARCHIVED.md`, `docs/fix-canales-color-rasterizado.md`, `docs/release-0.2.0.md`, `docs/release-0.2.1.md`.
- Rama actual: `development` (base `main`)
- Working tree: limpio

## Pendiente
- Verificación visual en Niri del resto: tema claro/oscuro, los 17 iconos, markdown de las notas, `Esc` cierra la proyección.
- Editor (follow-ups): undo/redo, navegación por palabras, panel de notas redimensionable.

## Limitaciones conocidas
- crates.io: no publicable mientras `gpui`/`gpui_platform` sean deps git de Zed. Job `publish` en `continue-on-error: true`.

## Metodología / reglas
SDD (OpenSpec) + TDD (RED→GREEN→REFACTOR). Gitflow: ramas desde `development`, todo por PR, commits `<emoji> <tipo>(<scope>): mensaje`. Delegar la escritura en subagentes.
