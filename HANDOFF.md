# Handoff — typshow — 2026-11-13

## Objetivo
Migrar la UI de Typshow de `egui`/`eframe` a `GPUI` (issue #12). **Migración terminada y mergeada.**

## Estado
- **Fases 1-7 completadas.** `typshow` es el único binario (GPUI); `egui`, `eframe`,
  `egui-phosphor`, `winit` y `parking_lot` retirados de `Cargo.toml`.
- Change `migracion-gpui` **archivado** (`openspec/changes/archive/2026-11-13-migracion-gpui/`);
  spec viva en `openspec/specs/gpui-ui/spec.md`.
- `plans/PLAN-002-ARCHIVED.md` cerrado. `docs/migracion-gpui.md` reescrito;
  `docs/gpui-spike.md` marcado como registro histórico.
- Verificación en verde: `cargo build`, `cargo build --release`, `cargo test` (166 lib + 2
  integración), `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`.
- **Todo mergeado en `development`** vía PR #19 (`dbe296f`). No hay trabajo pendiente sin commitear.

## Ramas
- `feature/gpui-migration` (PR #14) y `feature/gpui-spike` (PR #13): **mergeadas**, siguen
  existiendo en local y en `origin` → **borrables**.
- `feature/gpui-adwaita-theme` (PR #18), `feature/gpui-decouple-notes` (PR #16) y
  `feature/gpui-default-binary` (PR #19): **mergeadas y ya borradas** (no aparecen en `git branch -a`).
- `chore/handoff-task-2-2` (PR #17) sigue **abierto** y obsoleto: su contenido ya está en
  `development`. Cerrar sin mergear.

## Pendiente
- **Verificación visual en Niri**: tema, iconos, markdown de las notas y Esc/proyección.
- Follow-ups del editor: undo/redo, navegación por palabras, panel de notas redimensionable,
  decidir `MdElement::EmptyLine` (sin productor).

## Siguiente acción
Abrir en Niri `cargo run -- <fichero.typ>` y comprobar: tema claro/oscuro, los 17 iconos,
el markdown de las notas y que `Esc` cierra la proyección.

## Riesgos y notas
- Para el binario definitivo usar `cargo build --release`.
- El clippy del runner es más nuevo que el local: verificar lints nuevos con
  `rustup run 1.99.1 cargo clippy --all-targets -- -D warnings`.
