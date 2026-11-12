# PLAN-001 — Spike: Migración de la UI a GPUI

- **Issue:** #12 (Migrar la interfaz a GPUI) — https://github.com/atareao/typshow/issues/12
- **Tipo:** Spike / prueba de concepto (no migración completa)
- **Branch:** `feature/gpui-spike` (desde `development`)
- **Estado:** COMPLETADO (spike verificado; ver `docs/gpui-spike.md`)

## Objetivo

Evaluar la viabilidad de sustituir `egui`/`eframe` por **GPUI** en Typshow, con dos ventanas nativas (presenter + audiencia fullscreen) sincronizadas, antes de comprometer una migración completa.

## Contexto de arquitectura actual

| Fichero | LOC | Rol |
|---|---|---|
| `src/presenter.rs` | 454 | Ventana controlador egui: Start/Prev/Next/End, toggle tema, toggle viewport |
| `src/fullscreen.rs` | 114 | Ventana audiencia a pantalla completa |
| `src/render.rs` | 275 | `typst_render::render()` → `tiny_skia::Pixmap` → `ColorImage` → `ctx.load_texture()` |
| `src/app.rs` | 168 | `AppState` + `SharedState = Arc<Mutex<AppState>>` |
| `src/theme.rs` | 104 | Overrides visuales Adwaita/GTK |

**Punto clave de integración:** GPUI necesita `RenderImage` construido desde `Frame::new(RgbaImage)` y almacena los frames en **BGRA**, por lo que el pixmap RGBA de Typst requiere un swap de canales R/B.

## Hallazgos de la investigación (estado de GPUI)

- GPUI **no está publicado oficialmente** en crates.io. Opciones: builds no oficiales (`gpui-unofficial` 1.27, `gpui-x` 0.2.2, `fc-gpui`, `gpui-fast`, `gpui-kit`) o dependencia git desde `zed-industries/zed`.
- Multi-window nativo: `cx.open_window(WindowOptions, |window, cx| ...)` + `WindowBounds::Fullscreen`. Cada ventana posee su propio `Window`/`Context`; el estado se comparte vía `Entity` y `cx.observe(...)`.
- Imágenes: `img(Arc<RenderImage>)`; `RenderImage::new(vec![Frame::new(rgba_image)])`.
- No existe equivalente a `egui-phosphor`; la configuración de fuentes difiere.
- Framework pre-1.0 con breaking changes frecuentes entre versiones.

## Tareas

- [x] T1. Crear branch `feature/gpui-spike` desde `development`.
- [x] T2. Añadir `gpui` + `gpui_platform` (features `wayland`,`x11`) como **binario separado** (`src/bin/gpui_spike.rs`) para no romper el build egui actual.
- [x] T3. Abrir dos ventanas GPUI: presenter (windowed 800×600) + audiencia (`Fullscreen`). Validar en Niri/Wayland.
- [x] T4. Estado compartido vía `Entity` (página actual, total, dark_mode) sincronizado entre ventanas.
- [x] T5. Pipeline Typst → RGBA → `RenderImage` (BGRA) y mostrar con `img()`.
- [x] T6. Controles Start/Prev/Next/End + atajos de teclado; repaint de la audiencia al cambiar de página.
- [x] T7. Theme mínimo dark/light (paridad Adwaita completa fuera de alcance).
- [x] T8. Escribir `docs/gpui-spike.md` con hallazgos y veredicto (migrar / no migrar).
- [x] T9. `cargo build` y `cargo test` siguen verdes.

## Fuera de alcance

- Edición de notas en la UI.
- Paridad total de theme/Adwaita.
- Empaquetado y release.

## Riesgos

- Crate no oficial y con breaking changes; puede requerir pin de commit.
- Fullscreen en Wayland con Niri: comportamiento de `WindowBounds::Fullscreen` y selección de monitor.
- Gestión de foco entre dos ventanas.
- Fuentes e iconos: no hay `egui-phosphor`.

## Criterio de éxito

Prototipo ejecutable que abra ambas ventanas, renderice una página Typst y navegue con sincronización, más un informe de viabilidad con veredicto razonado.