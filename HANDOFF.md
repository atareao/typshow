# Handoff — typshow — 2026-11-12 20:15

## Objetivo
Migrar la UI de Typshow de `egui`/`eframe` a `GPUI` (issue #12) por fases. **Ticket 3.12/M4 cerrado**: `EntityInputHandler` completo en la ventana del presentador (selección teclado/ratón/IME y portapapeles).

## Estado
- Hecho: Fases 1 y 3, 3.8, 3.9, 4.4, 2.4, 2.5, 3.10, 3.11 y M3 — testeadas.
- Hecho: **3.12/M4 completo**, todos los subítems (a) handler, (b) render, (c) ratón, (d) portapapeles, (e) teclado y (f) tests — implementados, testeados y auditados (`rust-reviewer`: APROBADO en (d); (a)(b)(c)(e) ya lo estaban).
- Ítem (d): `copy_clipboard(target, cut, window, cx)` en `src/gpui_ui/presenter.rs`; Ctrl+C escribe la selección con `cx.write_to_clipboard(ClipboardItem::new_string(..))` y no muta; Ctrl+X copia + `replace_selection("")` + `commit_text` (marca `dirty` en la nota). No-op con selección vacía.
- Modelo puro `src/gpui_ui/edit.rs` (`TextEdit`, untracked), con tests.
- Rama actual: `feature/gpui-migration` (base: `development`)
- Working tree (sin commits): `Cargo.toml`, `Cargo.lock`, `HANDOFF.md`, `docs/migracion-gpui.md`, `openspec/changes/migracion-gpui/specs/gpui-ui/spec.md`, `openspec/changes/migracion-gpui/tasks.md`, `plans/PLAN-002.md`, `src/gpui_ui/{audience,deck,keys,mod,presenter}.rs`; untracked: `src/gpui_ui/edit.rs`. HEAD = `bd34f21`.
- PRs: **#13** (spike) mergeado; **#14** (migración) borrador → `development`.

Verificación vigente: `cargo test --lib` = **164 passed**; `cargo build --bin typshow-gpui` OK; `cargo fmt --check` limpio; clippy bin sin warnings en `src/gpui_ui/presenter.rs` (baseline 22 en la lib `src/`).

## Pendiente
3.12/M4 está cerrado, así que la decisión «NO commitear hasta cerrar M4» queda levantada. Próxima tarea:
1. Probar a mano en Niri (Wayland) con el binario `typshow-gpui`: selección por clic/arrastre (ítem c), Ctrl+C/X/V y acentos/IME.
2. Decidir el borrado del código legacy ya sin cablear en `src/gpui_ui/keys.rs` (`apply_edit`, `insert_at`, `backspace_at`, `delete_at`, `display_with_caret`) — documentado como candidato en `docs/migracion-gpui.md`.
3. Commit(s) gitmoji en `feature/gpui-migration` y actualizar el PR #14 (borrador → listo para revisión) hacia `development`.

## Punto de partida
```
cd /data/rust/typshow
git checkout feature/gpui-migration    # ya es la rama actual; NO crear otra
# (los cambios están sin commitear: no tocar el working tree con checkout/pull)
cargo test --lib                       # baseline 164 verdes
cargo build --bin typshow-gpui
```
Referencias: `src/gpui_ui/presenter.rs` (`handle_edit_key`, `copy_clipboard`, `paste_clipboard`, `TextEdit::selected_text`), `src/gpui_ui/edit.rs`, spec `openspec/changes/migracion-gpui/specs/gpui-ui/spec.md` (requisito «Edición de texto con selección e IME»).

## Metodología / reglas
SDD (OpenSpec: spec antes que código) + TDD (RED→GREEN→REFACTOR con `cargo test`/`cargo fmt --check`/`cargo clippy`). Gitflow con gitmoji: `feature/*` desde `development`, PR a `development`. Delegar la escritura de código a subagentes (nota: el subagente `rust-writer` falló en esta sesión, terminó sin cambios y sin respuesta; se usó `backend-dev`).

## Riesgos y notas
- El clic/arrastre del ítem (c) está testeado headless pero **falta probarlo a mano en Niri** (Wayland) con `typshow-gpui`.
- Residuo MENOR teórico del ítem (c): si se cierra un editor mientras el otro sigue editándose durante un arrastre, `is_selecting` no se resetea (inocuo por la guarda de botón pulsado).
- `write_back` de `Heading` nunca marca `dirty` (persiste con `save_heading` en Enter) — es intencional; el scenario del spec ya lo acota al cuerpo de la nota.
- Prerequisito de build: `Cargo.toml` usa dev-dependency `gpui` con feature `test-support`; GPUI es git-dependency rev `a1f2e21`.
