# Handoff — typshow — 2026-11-13 09:19

## Objetivo
Publicar la release 0.2.0 de Typshow tras completar la migración de la UI de egui a GPUI (issue #12, ya cerrado).

## Estado
- Hecho: migración GPUI completa (fases 1-7), mergeada en `development` vía PRs #14, #16, #18, #19.
- Hecho: `typshow` es el único binario; `egui`/`eframe`/`egui-phosphor`/`winit`/`parking_lot` retirados de `Cargo.toml`.
- Hecho: change `migracion-gpui` archivado en `openspec/changes/archive/2026-11-13-migracion-gpui/`; spec viva en `openspec/specs/gpui-ui/spec.md`.
- Hecho: `plans/PLAN-002-ARCHIVED.md`, `docs/migracion-gpui.md` reescrito, `docs/gpui-spike.md` marcado como histórico, `HANDOFF.md` (PR #20).
- Hecho: issue #12 cerrado; PRs #13 y #17 cerrados.
- Hecho: verificación en verde — 166 tests de lib + 2 de integración, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo build` y `cargo build --release`.
- En curso: nada. La release 0.2.0 está decidida pero no ejecutada.
- Rama actual: `development` (base: `main`)
- Working tree: limpio

## Pendiente
Publicar la release **0.2.0**: el workflow `release-prepare.yml` aplica `--major` por el `!:` del commit `f97a318`, así que hay que forzar el minor antes de mergear `development` → `main`.

## Punto de partida
```bash
git checkout development && git pull origin development
git checkout -b chore/release-0.2.0
# ajustar la regla de bump en .github/workflows/release-prepare.yml para 0.x
#   (breaking en 0.x => --minor) o fijar current_version: 0.2.0 en .vampus.yml
vampus preview --minor   # debe dar 0.2.0
cargo test && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings
# commit gitmoji, push, PR a development, merge
# después PR development -> main: el CI hace bump, CHANGELOG, tag v0.2.0 y publica
```

## Metodología / reglas
SDD (OpenSpec) + TDD (RED→GREEN→REFACTOR). Rama `feature/*` o `chore/*` desde `development`; nunca commits directos a `main` ni `development`; todo entra por PR a `development`. Commits `<emoji> <tipo>(<scope>): mensaje`. Delegar la escritura en subagentes y auditar con `rust-reviewer` antes de cerrar.

## Riesgos y notas
- **Decisión del usuario**: versión 0.2.0 (minor), no 1.0.0. Si se mergea a `main` sin tocar el workflow, se publica 1.0.0 en crates.io.
- `release-prepare.yml` sincroniza `development` con `main` al final; verificar que no pisa trabajo en curso.
- Verificación visual en Niri pendiente: tema claro/oscuro, los 17 iconos, markdown de las notas y `Esc` cerrando la proyección.
- Follow-ups del editor (sin issue abierto): undo/redo, navegación por palabras, panel de notas redimensionable, `MdElement::EmptyLine` sin productor.
- Ramas mergeadas y borrables: `feature/gpui-migration`, `feature/gpui-spike` (en local y en `origin`).
- El clippy del runner es más nuevo que el local: verificar con `rustup run 1.99.1 cargo clippy --all-targets -- -D warnings`.
