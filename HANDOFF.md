# HANDOFF — Typshow

## Estado
Migración de la UI de Typshow de `egui`/`eframe` a **GPUI** (issue #12), por fases.
- **Fase 1** (estructura + núcleo + presenter/audiencia básicos): COMPLETADA.
- **Fase 3** (presenter: abrir fichero, proyección, notas, navegación, clic, Esc): COMPLETADA y auditada.
- **3.8** edición del título de la nota: COMPLETADA.
- **4.4** audiencia en monitor secundario: COMPLETADA (con limitación documentada).
- **3.9** editor de notas con cursor (flechas/Home/End/Backspace/Delete + pegar Ctrl+V): COMPLETADA (tests).
- PRs: **#13** (spike, mergeado), **#14** (migración, borrador) → `development`.

## Rama actual
`feature/gpui-migration`. Coexisten `typshow` (egui, intacto) y `typshow-gpui` (GPUI).

## Hecho
- `src/gpui_ui/`: `deck.rs` (estado + render + notas + fichero + proyección),
  `keys.rs` (teclado por `on_key_down`; navegación + edición con cursor),
  `notes.rs` (campos `cursor`/`heading_cursor`),
  `presenter.rs`, `audience.rs`, `theme.rs`, `mod.rs`.
- Panel de notas con edición del **título** (`start_edit_heading`/`save_heading`).
- `app_id` por ventana: `com.atareao.typshow.presenter` y
  `com.atareao.typshow.audience`, para reglas de compositor.
- Doc: `docs/migracion-gpui.md` (incluye la regla de Niri `open-on-output`).
- Editor de notas/título con **cursor** (offset en bytes): inserción en el cursor,
  `←`/`→`/`Home`/`End`, `Backspace`/`Delete`, `Enter`, `Esc` y pegar `Ctrl+V`.
  Operaciones puras en `keys.rs` (`prev_boundary`, `insert_at`, `backspace_at`,
  `delete_at`, `display_with_caret`) con tests.
- Verificado: `cargo build`/`test` (89 lib + 1 spike + 2 pipeline), clippy limpio.

## Limitación conocida (4.4)
GPUI solo respeta `display_id` en X11; en Wayland/Niri la ventana de audiencia se
abre en la salida activa. Solución: regla de Niri sobre `app_id` (ver
`docs/migracion-gpui.md`).

## Hallazgo (acentos / IME) — verificado
El backend **Wayland** de GPUI inicializa `xkb::compose::State`
(`gpui_linux/.../wayland/client.rs:2092`) y compone las teclas muertas: al
teclear `´`+vocal entrega un `KeyDown` normal con `key="aacute"`,
`key_char=Some("á")`. Como `keys.rs` inserta `key_char`, **los acentos ya
funcionan** en los editores de nota y título. El `EntityInputHandler` (M4) NO es
imprescindible para acentos; solo aporta cursor/selección/ratón/pegar y el
pre-edit visual de la tecla muerta.
- Cobertura: tests puros `note_edit_action` / `heading_edit_action` con
  `aacute`/`ntilde`/`oacute` (10 tests nuevos; total lib 83).
- Nota: el clic automatizado con `ydotool` dispara el overview de niri
  (el `mousemove` se interpreta como gesto); la verificación E2E de tecleo hay
  que hacerla a mano.

## Siguiente
1. **M4 (resto)** editor con `EntityInputHandler`: selección (Shift+flechas/ratón),
   resaltado y *pre-edit* de IME. (Los acentos ya funcionan sin él.)
2. **M3** `rfd` asíncrono + compilación Typst fuera del hilo de UI.
3. **2.2** desacoplar `Notes` de egui (`Notes::draw` usa egui).
4. Fase 5 (tema Adwaita + iconos), Fase 6 (GPUI por defecto, retirar egui), Fase 7 (cierre).

## Pendiente de verificación manual
El editor con cursor no se pudo probar E2E automatizado (el `mousemove` de
`ydotool` dispara el overview de niri y el clic no llega). Conviene abrir la app,
pulsar **Notas → Editar** y comprobar: flechas mueven el cursor, `|` visible,
`Ctrl+V` pega, acentos OK.

## Comandos útiles
- Run: `cargo build --bin typshow-gpui && ./target/debug/typshow-gpui <file.typ>`
- Matar: `pkill -x typshow-gpui` (NO `pkill -f`).
- Tests: `cargo test` · Lint: `cargo clippy --bin typshow-gpui`
