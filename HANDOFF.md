# Handoff — typshow — 2026-11-13 (release 0.2.0 publicada)

## Objetivo
Publicar la release 0.2.0 (primera con la UI en GPUI). **Publicada.**

## Estado
- Hecho: tag `v0.2.0`, `main` en `0.2.0`, `CHANGELOG.md` y GitHub Release `v0.2.0` con binarios x86_64 y aarch64.
- Hecho: `development` sincronizado con `main`.
- Hecho: `release-prepare.yml` corregido (breaking en 0.x → `--minor`; detección de `feat` con gitmoji).
- Hecho: `plans/PLAN-003-ARCHIVED.md`, `docs/release-0.2.0.md`, issue #21 cerrado.
- Hecho: PR #24 mergeado (permisos `actions: write` + publish no bloqueante), ya en `development`.
- Rama actual: `development` (base `main`)
- Working tree: limpio

## Pendiente
- Nada crítico. Follow-ups: verificación visual en Niri (tema claro/oscuro, los 17 iconos, markdown de las notas, `Esc` cierra la proyección); editor (undo/redo, navegación por palabras, panel de notas redimensionable).

## Limitaciones conocidas
- crates.io: no publicable mientras `gpui`/`gpui_platform` sean deps git de Zed. Job `publish` en `continue-on-error: true`.
- El paso "Trigger release workflow" ya lleva `actions: write` en `development`; entra en vigor en el próximo merge a `main` (hasta entonces, `release.yml` se dispara a mano).

## Metodología / reglas
SDD (OpenSpec) + TDD (RED→GREEN→REFACTOR). Gitflow: ramas desde `development`, todo por PR, commits `<emoji> <tipo>(<scope>): mensaje`. Delegar la escritura en subagentes.
