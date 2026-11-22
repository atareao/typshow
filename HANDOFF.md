# HANDOFF — Typshow

## Estado actual

Rama `feature/guion-embebido-en-typ` con **PR #53** abierta a `development`
(pendiente de merge). `cargo test` 228 lib + 3 integración verdes, clippy
(`-D warnings`) y fmt limpios. Auditoría `@rust-reviewer`: APROBADO.

## Completado (sesión 2026-11-22)

### Issue #52 — Guion embebido en el `.typ` como notas del presentador
- Bloques `// >>> typshow:notes` … `// >>> typshow:notes:end`, invisibles en el PDF
  (Typst no renderiza comentarios) y visibles solo en typshow.
- `notes_db.rs`: parser posicional (anclaje por `EmbeddedBlock::heading_line`),
  sembrado desde bloques con *fallback*, esquema `source_hash` (FNV-1a 64) +
  `edited`/`heading_edited`, `sync_from_blocks` selectivo no destructivo,
  `export_to_typ`/`render_exported_typ`, conversor `embed_script_into_typ`.
- `notes.rs`: `reimport` sin `clear_all`; `main.rs`: `typshow guion-embed <guion.md> <pres.typ>`.
- UI: `Deck::export_notes` + botón «Exportar» (`icons::UPLOAD_SIMPLE`); `deck::reimport_notes`
  guarda la edición en curso.
- 25+11 tests nuevos, incluido uno pixel a pixel de que el bloque no altera el PDF.
- Spec `openspec/changes/guion-embebido-en-typ/` · Plan `plans/PLAN-010.md` ·
  Doc `docs/guion-embebido.md`.

### Completado (sesión 2026-11-15)

### Issue #42 — arranque solo con el presentador, cierre conjunto y tema
- `Deck::empty()` → `projection: false`; `Deck::no_document()` cablea el tema;
  `main.rs` `on_window_closed` → `quits_app_on_close` → `cx.quit()`. (Release v0.4.0.)

### Issue #46 — botón «Salir» y atajo Ctrl+Q
- `keys::quit_shortcut(key, control)` (predicado puro; ignora Shift/Alt/Super a propósito).
- `icons::SIGN_OUT` = `'\u{E42A}'` (en `ALL` y tabla dorada).
- `PresenterView::exit_app` = `window.remove_window()` + `cx.quit()`; botón `btn-salir`
  en la barra superior; `dispatch_key` gestiona `Ctrl+Q` antes de editar/navegar.
- Tests headless: Ctrl+Q, Ctrl+Q en edición, y clic del botón con proyección activa.

### ⬆️ Dependencias actualizadas
- `rfd` 0.15.4 → 0.18.0, `fontdb` 0.23.0 → 0.24.0, `pulldown-cmark` 0.12.2 → 0.13.6,
  `rusqlite` 0.32.1 → 0.42.0 (bundled). Sin cambios de código. `gpui` sin cambio de rev.

### Release
- **v0.5.0** publicada (PR #50 `development` → `main`; incluye #46 y las deps).

## Próximo / tarea pendiente

- **PR #53**: verificación manual en GUI sobre el episodio real (PLAN-010 §7.3) y
  `openspec archive guion-embebido-en-typ` al mergear.
- Probar de verdad el guion embebido en el episodio «El día que decidí montar mi red
  privada» (copias ya generadas en `/tmp/opencode/e2e/`).

## Follow-ups no bloqueantes

- Editor: undo/redo, navegación por palabras, panel de notas redimensionable.
- Rama `feature/sync-renderer` (1 commit, sin PR): decidir PR o borrar.
- Change OpenSpec `gpui-spike` sigue activo (spike, no archivado).
- Valorar si `Notes::set_heading` (sin llamantes tras `save_user_heading`) sigue en la API pública.
- `Ctrl+Q` solo en el presentador; valorar extenderlo a la ventana de audiencia.
- `fontdb` va duplicado en el binario (0.24 directa + 0.23 transitiva de
  `cosmic-text`/`typst-library`); se resolverá cuando esas crates suban.

## Notas de reproducción / entorno

- Lanzar: `target/debug/typshow <path.typ>` o sin argumento (solo presentador).
- Conversor: `target/debug/typshow guion-embed <guion.md> <presentacion.typ>` (idempotente).
- `ydotool key` usa **keycodes numéricos** (Esc=1, t=20, q=16, ctrl=29).
  `Ctrl+Q` = `ydotool key 29:1 16:1 16:0 29:0`. El ratón (`mousemove`/`click`) NO
  funciona en esta sesión.
- Captura: `grim -o DP-1 <png>`. Overview de Niri: `niri msg action toggle-overview`.
- `app_id`: presentador `com.atareao.typshow.presenter`, audiencia
  `com.atareao.typshow.audience`.
- CI clippy = **1.100.0**: `rustup run 1.100.0 cargo clippy --all-targets -- -D warnings`.
