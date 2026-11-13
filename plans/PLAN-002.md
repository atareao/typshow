# PLAN-002 — Migración de la UI a GPUI

- **Issue:** #12 (Migrar la interfaz a GPUI) — https://github.com/atareao/typshow/issues/12
- **Tipo:** Migración de framework de UI (por fases)
- **Change OpenSpec:** `openspec/changes/migracion-gpui/`
- **Branch:** `feature/gpui-migration`
- **Estado:** EN CURSO — **fases 1-6 completadas**. GPUI es el framework de UI y `typshow` el único binario: se retiró la app egui (`src/app.rs`, `src/presenter.rs`, `src/fullscreen.rs`, `src/theme.rs`, `src/notes_egui.rs`, `src/bin/gpui_spike.rs` y `src/bin/typshow_gpui.rs`) y se quitaron `egui`, `eframe`, `egui-phosphor` y `winit` de `Cargo.toml`. Paridad funcional **cerrada**: Esc cierra la proyección (con el bug de reentrada al cerrar la ventana arreglado), markdown de las notas con formato inline (negrita/código), título de ventana atado al documento, tema Adwaita e iconos Phosphor embebidos (M3/M4: render/apertura asíncrona, `rfd` no bloqueante, editor de notas con `EntityInputHandler` — selección teclado/ratón, IME y portapapeles). Pendiente: **fase 7** (`docs/migracion-gpui.md` + verificación final `cargo build`/`test`/`clippy`) y la **verificación visual en Niri** del tema, los iconos y la proyección.
- **Precedente:** spike `gpui-spike` (PR #13); veredicto de viabilidad en `docs/gpui-spike.md`

## Objetivo

Convertir **GPUI en el framework de UI por defecto** de Typshow, sustituyendo
`egui`/`eframe` con **paridad funcional** y sin dejar la app inservible durante el proceso.

## Estado actual

| Fichero | LOC | Rol |
|---|---|---|
| `src/main.rs` | 80 | Arranque GPUI: fuente Phosphor, tracing y ventana del presentador |
| `src/gpui_ui/presenter.rs` | 4314 | `PresenterView`: nav, notas (markdown inline), actual+siguiente, proyección y título |
| `src/gpui_ui/deck.rs` | 872 | `Deck`: `TypstWorld` + caché de `RenderImage`, apertura asíncrona |
| `src/gpui_ui/audience.rs` | 105 | Ventana de audiencia (fullscreen) |
| `src/gpui_ui/theme.rs` | 411 | Tema Adwaita/GTK4 en tokens |
| `src/gpui_ui/icons.rs` | 212 | Iconos Phosphor embebidos |
| `src/render.rs` | 198 | `TypstWorld` (typst → documento); sin la parte egui |
| `src/notes.rs` | 449 | Notas + Markdown (lógica compartida) |
| `src/notes_db.rs` | 749 | SQLite de notas (WAL) |

Retirados en la fase 6: `src/app.rs`, `src/presenter.rs`, `src/fullscreen.rs`, `src/theme.rs`, `src/notes_egui.rs`, `src/bin/gpui_spike.rs` y `src/bin/typshow_gpui.rs`.

## Estrategia

**Coexistencia por binarios**: durante la migración convivieron `typshow` (egui) y
`typshow-gpui` (nuevo). Alcanzada la paridad, GPUI pasó a ser `typshow` y se retiró egui
(fase 6). Se reutiliza el dominio (`TypstWorld`, `Notes`, `NotesDb`) y solo se reescribió
la capa de UI.

## Fases

1. **Preparación y estructura** — `src/gpui_ui/` + binario GPUI coexistiendo. ✅
2. **Núcleo** — `Deck` con `TypstWorld` y caché de `RenderImage`; reutilizar `Notes`/`NotesDb`; manejo de errores. ✅
3. **Presenter** (✅) — paneles Actual+Siguiente, barra de navegación, abrir fichero (`rfd`), toggle de proyección, toggle de tema, panel de notas (markdown con formato inline), atajos y clic.
4. **Audiencia** (✅) — fullscreen, sincronización, atajos/clic/Esc (Esc cierra la proyección), monitor secundario (Niri).
5. **Tema e iconos** — estilo Adwaita en GPUI (**5.1 ✅** tema Adwaita/GTK4 con tokens únicos — paleta de 21 colores + constantes de geometría, 8 tests de contraste WCAG, clippy/fmt/tests en verde; **5.2 ✅** iconos Phosphor embebidos — TTF vendorizado en `assets/fonts/`, registrado con `add_fonts` y glifo pintado con `font_family("Phosphor")` para no depender del fallback del sistema; 17 controles con icono y 5 tests (tabla dorada de codepoints + cmap de la fuente embebida); **verificación visual en Niri pendiente**). Crate `egui-phosphor` retirado en la fase 6.
6. **Framework por defecto** (✅) — `typshow` = GPUI (único binario); retirados `egui`, `eframe`, `egui-phosphor` y `winit`. `src/main.rs` es el punto de entrada GPUI y la app egui (`src/app.rs`, `src/presenter.rs`, `src/fullscreen.rs`, `src/theme.rs`, `src/notes_egui.rs`) se ha eliminado.
7. **Cierre** (⏳ pendiente) — `docs/migracion-gpui.md`, verificación (`cargo build`/`test`/`clippy`).

## Fuera de alcance

- Funcionalidad nueva más allá de la paridad.
- Cambios en el pipeline de render Typst (se reutiliza).

## Criterio de éxito

GPUI es el framework por defecto de `typshow`, con **paridad funcional** completa (doble
ventana sincronizada, render, navegación, notas, tema e iconos), `cargo build`/`test`/`clippy`
en verde y egui retirado.

## Riesgos

- GPUI pre-1.0 con dependencia git (rev fijado) → actualizaciones manuales; posibles breaking changes.
- Compilación pesada del árbol de Zed → caché de CI; loop de desarrollo más lento.
- Sin `egui-phosphor` → embeber Phosphor (TTF) o migrar a SVG.
- Fullscreen / monitor secundario en Wayland/Niri → validar; conservar `niri msg` si aplica.
- Regresión si se cambia el binario por defecto antes de la paridad → mitigado por la coexistencia.