# HANDOFF — Typshow

## Estado
Migración de la UI de Typshow de `egui`/`eframe` a **GPUI** (issue #12), por fases.
- **Fase 1** (estructura + núcleo + presenter/audiencia básicos): COMPLETADA.
- **Fase 3** (presenter: abrir fichero, proyección, notas, navegación, clic, Esc): COMPLETADA y auditada.
- PRs: **#13** (spike), **#14** (migración, borrador) → `development`.

## Rama actual
`feature/gpui-migration`. Coexisten `typshow` (egui, intacto) y `typshow-gpui` (GPUI).

## Hecho
- `src/gpui_ui/`: `deck.rs` (estado + render + notas + fichero + proyección),
  `keys.rs` (teclado por `on_key_down`), `presenter.rs`, `audience.rs`, `theme.rs`, `mod.rs`.
- Auditoría `@rust-reviewer` Fase 3 aplicada:
  - **Crítico**: `is_active` = foco, no "vivo" → arreglado con pertenencia en `cx.windows()`
    + suscripción `on_window_closed` (WeakEntity) para sincronizar `audience`/`projection`.
  - Banner de error visible; errores de notas propagados a `deck.error`.
- Verificado: `cargo build`/`test` (73 lib + 1 spike + 2 pipeline), clippy limpio en `gpui_ui`.

## Siguiente
1. **Editar título (heading)** de la nota (tarea 3.8).
2. **Audiencia en monitor secundario** (Niri) — tarea 4.4 (Fase 4).
3. **Desacoplar `Notes` de egui** (tarea 2.2): `Notes::draw` usa egui; `Deck` lo arrastra.
4. Mejoras de la auditoría: `rfd` asíncrono + compilación en background (M3);
   editor con `EntityInputHandler` para acentos/IME (M4).
5. Fase 5 (tema Adwaita + iconos), Fase 6 (GPUI por defecto, retirar egui), Fase 7 (docs/cierre).

## Comandos útiles
- Run: `cargo build --bin typshow-gpui && ./target/debug/typshow-gpui <file.typ>`
- Matar: `pkill -x typshow_gpui` (NO `pkill -f`).
- Tests: `cargo test` · Lint: `cargo clippy --bin typshow-gpui`
