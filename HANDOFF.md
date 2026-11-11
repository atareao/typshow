# HANDOFF — Typshow

## Estado
Migración de la UI de Typshow de `egui`/`eframe` a **GPUI** (issue #12), por fases.
- **Fase 1** (estructura + núcleo + presenter/audiencia básicos): COMPLETADA.
- **Fase 3** (presenter: abrir fichero, proyección, notas, navegación, clic, Esc): COMPLETADA y auditada.
- **3.8** edición del título de la nota: COMPLETADA.
- **4.4** audiencia en monitor secundario: COMPLETADA (con limitación documentada).
- PRs: **#13** (spike, mergeado), **#14** (migración, borrador) → `development`.

## Rama actual
`feature/gpui-migration`. Coexisten `typshow` (egui, intacto) y `typshow-gpui` (GPUI).

## Hecho
- `src/gpui_ui/`: `deck.rs` (estado + render + notas + fichero + proyección),
  `keys.rs` (teclado por `on_key_down`; navegación + edición de nota/título),
  `presenter.rs`, `audience.rs`, `theme.rs`, `mod.rs`.
- Panel de notas con edición del **título** (`start_edit_heading`/`save_heading`).
- `app_id` por ventana: `com.atareao.typshow.presenter` y
  `com.atareao.typshow.audience`, para reglas de compositor.
- Doc: `docs/migracion-gpui.md` (incluye la regla de Niri `open-on-output`).
- Verificado: `cargo build`/`test` (73 lib + 1 spike + 2 pipeline), clippy limpio en `gpui_ui`.

## Limitación conocida (4.4)
GPUI solo respeta `display_id` en X11; en Wayland/Niri la ventana de audiencia se
abre en la salida activa. Solución: regla de Niri sobre `app_id` (ver
`docs/migracion-gpui.md`).

## Siguiente
1. **M4** editor de notas con `EntityInputHandler` (acentos/IME) — importante para español.
2. **M3** `rfd` asíncrono + compilación Typst fuera del hilo de UI.
3. **2.2** desacoplar `Notes` de egui (`Notes::draw` usa egui).
4. Fase 5 (tema Adwaita + iconos), Fase 6 (GPUI por defecto, retirar egui), Fase 7 (cierre).

## Comandos útiles
- Run: `cargo build --bin typshow-gpui && ./target/debug/typshow-gpui <file.typ>`
- Matar: `pkill -x typshow-gpui` (NO `pkill -f`).
- Tests: `cargo test` · Lint: `cargo clippy --bin typshow-gpui`
