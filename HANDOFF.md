# HANDOFF — Typshow

## Estado actual
Release **0.3.0** publicada: tag `v0.3.0`, GitHub Release con binarios x86_64/aarch64;
`main` y `development` ambos en `0.3.0` y sincronizados. Los 3 changes OpenSpec archivados.
Nada pendiente de merge; todos los CI en verde.

## Completado (sesión 2026-11-14)
- Merge de los PRs #36 (tema), #37 (cursiva) y #38 (Esc) a `development`.
- Archivado OpenSpec (PR #39): `persistir-tema`, `markdown-cursiva`, `esc-cierra-proyeccion`
  → `openspec/changes/archive/2026-11-14-*`; `openspec/specs/gpui-ui/spec.md` actualizado.
- Release 0.3.0 (PR #40 `development`→`main`): bump **minor** (por el `feat` de tema),
  CHANGELOG, tag `v0.3.0`, binarios `x86_64`/`aarch64`.

## Tarea pendiente (relevo)
- Verificación visual en Niri: tema persistente tras reinicio, cursiva en las notas y `Esc`
  cierra la proyección con el editor abierto.
- (opcional/cosmético) marcar las casillas de los `tasks.md` de los changes archivados.

## Follow-ups no bloqueantes
- Editor: undo/redo, navegación por palabras, panel de notas redimensionable.
- Rama `feature/sync-renderer` (1 commit, sin PR): decidir abrir PR o borrar.
- Change OpenSpec `gpui-spike` sigue activo (spike, no archivado).

## Notas de reproducción / entorno
- Lanzar: `target/debug/typshow <path.typ>` (acepta `.typ`, no PDF). `pkill -x typshow` (no `pkill -f`).
- Captura: `grim -o DP-1 <png>`.
- Teclado en vivo: `export YDOTOOL_SOCKET=/run/user/$(id -u)/.ydotool_socket; ydotool key 1:1 1:0` (Esc).
- `app_id`: presenter `com.atareao.typshow.presenter`, audience `com.atareao.typshow.audience`.
- CI clippy = **1.100.0**: `rustup run 1.100.0 cargo clippy --all-targets -- -D warnings`.
- La app arranca con `projection = true` (la ventana de audiencia se abre sola al lanzar).
