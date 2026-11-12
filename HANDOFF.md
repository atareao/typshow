# Handoff — typshow — 2026-11-12 20:25

## Objetivo
Migrar la UI de Typshow de `egui`/`eframe` a `GPUI` (issue #12) por fases. **3.12/M4 cerrado y commiteado**; solo falta la verificación manual en Niri antes de sacar el PR de borrador.

## Estado
- Hecho y commiteado: Fases 1/3 (doble ventana, render asíncrono, caché LRU), 3.8–3.12/M4 (editor `TextEdit` + `EntityInputHandler`: selección teclado/ratón, IME y portapapeles Ctrl+C/X/V), 2.5 (apertura asíncrona de `.typ`).
- Hecho: borrado del editor legacy sin cablear de `src/gpui_ui/keys.rs` (se conservan `apply_nav` y helpers de frontera).
- Hecho: `tasks.md` 3.12 `[x]`, delta de spec y `docs/migracion-gpui.md` actualizados.
- Commits en `feature/gpui-migration` (pusheados a origin): `0216c2e` (editor M4 + 2.5 + test-support), `98c38e7` (borra legacy keys.rs), `0088352` (docs/spec/tasks/handoff). HEAD anterior era `bd34f21`.
- Working tree: limpio.
- PR **#14** (`feature/gpui-migration` → `development`): **OPEN en borrador**, título y body actualizados. URL: https://github.com/atareao/typshow/pull/14

Verificación vigente: `cargo test --lib` = **149 passed** (bajó de 164 al borrar 15 tests legacy); `cargo build --bin typshow-gpui` OK; `cargo fmt --check` limpio; clippy bin sin warnings nuevos en `src/gpui_ui/` (quedan ~22 warnings pre-existentes en la lib).

## Pendiente
Única tarea: **verificación manual en Niri (Wayland)** con el binario `typshow-gpui` y un `.typ` de prueba (no hay fixtures en el repo):
```
cargo run --bin typshow-gpui -- /ruta/a/presentacion.typ
```
Comprobar: (1) selección con clic/arrastre en el cuerpo de la nota y en el título; (2) resaltado y cursor; (3) Ctrl+C, Ctrl+X y Ctrl+V; (4) acentos/IME (p. ej. ñ, á, dead keys).
Tras pasar: marcar el PR #14 como **listo para revisión** (`gh pr ready 14`) y avisar para auditoría final/merge.

## Punto de partida
```
cd /data/rust/typshow
git checkout feature/gpui-migration && git pull
cargo test --lib                       # baseline 149 verdes
cargo run --bin typshow-gpui -- <fichero.typ>
```
Referencias: `src/gpui_ui/presenter.rs` (`handle_edit_key`, `copy_clipboard`, `paste_clipboard`), `src/gpui_ui/edit.rs` (`TextEdit`), `openspec/changes/migracion-gpui/specs/gpui-ui/spec.md` (req. «Edición de texto con selección e IME»).

## Metodología / reglas
SDD (OpenSpec) + TDD (RED→GREEN→REFACTOR con `cargo test`/`cargo fmt --check`/`cargo clippy`). Gitflow con gitmoji: `feature/*` desde `development`, PR a `development`. Delegar la escritura de código a subagentes.

## Riesgos y notas
- El clic/arrastre y el portapapeles están testeados headless; **falta la prueba real en Niri**.
- Residuo MENOR: si se cierra un editor mientras el otro sigue editándose durante un arrastre, `is_selecting` no se resetea (inocuo por la guarda de botón pulsado).
- `write_back` de `Heading` nunca marca `dirty` (persiste con `save_heading` en Enter) — intencional; el scenario del spec está acotado al cuerpo.
- El subagente `rust-writer` falló esta sesión (terminaba sin cambios ni respuesta); se usó `backend-dev`.
- Prerequisito de build: dev-dependency `gpui` (feature `test-support`); GPUI es git-dependency rev `a1f2e21`.
