# Handoff — typshow — 2026-11-12 20:25

## Objetivo
Migrar la UI de Typshow de `egui`/`eframe` a `GPUI` (issue #12) por fases. **3.12/M4 cerrado, verificado en Niri y con el PR #14 listo para revisión.**

## Estado
- Hecho y commiteado: Fases 1/3 (doble ventana, render asíncrono, caché LRU), 3.8–3.12/M4 (editor `TextEdit` + `EntityInputHandler`: selección teclado/ratón, IME y portapapeles Ctrl+C/X/V), 2.5 (apertura asíncrona de `.typ`).
- Hecho: borrado del editor legacy sin cablear de `src/gpui_ui/keys.rs` (se conservan `apply_nav` y helpers de frontera).
- Hecho: `tasks.md` 3.12 `[x]`, delta de spec y `docs/migracion-gpui.md` actualizados.
- Commits en `feature/gpui-migration` (pusheados a origin): `0216c2e` (editor M4 + 2.5 + test-support), `98c38e7` (borra legacy keys.rs), `0088352` (docs/spec/tasks/handoff). HEAD anterior era `bd34f21`.
- Hecho: perfil dev optimizado en `Cargo.toml` (`[profile.dev.package."*"] opt-level = 3`): el render de página en debug baja de ~0.7-0.9 s a ~20 ms (commit `6a6ff7f`).
- Working tree: limpio.
- PR **#14** (`feature/gpui-migration` → `development`): **listo para revisión** (revisión manual en Niri ✅: selección, Ctrl+C/X/V, acentos/IME). URL: https://github.com/atareao/typshow/pull/14

Verificación vigente: `cargo test --lib` = **149 passed** (bajó de 164 al borrar 15 tests legacy); `cargo build --bin typshow-gpui` OK; `cargo fmt --check` limpio; clippy bin sin warnings nuevos en `src/gpui_ui/` (quedan ~22 warnings pre-existentes en la lib).

## Pendiente
- **Siguiente paso inmediato:** revisión y merge del **PR #14**.
- **Trabajo restante de la migración:**
  1. **2.2** — desacoplar `Notes` de egui.
  2. **Fases 5-7** — tema/iconos, GPUI por defecto y cierre de la migración.

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
- El clic/arrastre y el portapapeles se verificaron OK en Niri (Wayland).
- Residuo MENOR: si se cierra un editor mientras el otro sigue editándose durante un arrastre, `is_selecting` no se resetea (inocuo por la guarda de botón pulsado).
- `write_back` de `Heading` nunca marca `dirty` (persiste con `save_heading` en Enter) — intencional; el scenario del spec está acotado al cuerpo.
- El subagente `rust-writer` falló esta sesión (terminaba sin cambios ni respuesta); se usó `backend-dev`.
- Prerequisito de build: dev-dependency `gpui` (feature `test-support`); GPUI es git-dependency rev `a1f2e21`.
- Rendimiento: el cuello de botella era el build de *debug* (render ~35-44x más lento). Con el perfil dev optimizado ya va fino; para el binario definitivo usar `cargo build --release`. Si aún se percibe latencia en el panel *Siguiente*, considerar prefetch de la página +2.
