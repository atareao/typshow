# Design

## Context

Ver `proposal.md` - Why. La app actual usa `eframe` con `ctx.show_viewport_immediate`
para dos viewports, `SharedState = Arc<Mutex<AppState>>` y render Typst →
`tiny_skia::Pixmap` RGBA → `ctx.load_texture`. El spike GPUI (PR #13) ya resuelve los
bloques técnicos: dos `cx.open_window`, estado `Entity<Deck>` con `cx.observe(...)`, y
render a `RenderImage` (RGBA **premultiplicado** → BGRA **no premultiplicado**).

## Goals / Non-Goals

**Goals:**
- Migrar toda la funcionalidad de UI a GPUI con **paridad funcional**.
- Dejar GPUI como framework por defecto, manteniendo la app usable durante la migración.
- Reutilizar la lógica no-UI (`AppState`, `TypstWorld`, `Notes`, `NotesDb`).

**Non-Goals:**
- Funcionalidad nueva más allá de la paridad.
- Cambiar el pipeline de render Typst (se reutiliza).

## Decisions

- **Coexistencia por binarios** (`typshow` egui + `typshow-gpui` GPUI) frente a big-bang:
  permite comparar y volver atrás; al cerrar la migración GPUI toma el nombre `typshow`.
- **Reutilizar el dominio**: `AppState` (navegación), `TypstWorld` (render) y
  `Notes`/`NotesDb` se conservan; solo se reescribe la capa de UI (`Notes::draw` y
  `Renderer::get_page` son lo único específico de egui).
- **`Entity` compartida** (frente a `Arc<Mutex>`): patrón idiomático de GPUI; cada ventana
  observa la entidad y repinta (`cx.observe` + `cx.notify`).
- **Render**: pixmap RGBA premultiplicado → BGRA no premultiplicado
  (`swap_rgba_pa_to_bgra`) → `RenderImage::new(vec![Frame::new(RgbaImage)])` → `img()`.
- **Estructura de módulos**: `src/gpui/` con `app.rs` (estado/`Deck`), `presenter.rs`,
  `audience.rs`, `theme.rs`, `icons.rs`.

## Risks / Trade-offs

- **GPUI pre-1.0 con dependencia git** → pin de `rev`; actualizaciones manuales y posibles
  breaking changes.
- **Compilación pesada del árbol de Zed** → caché de CI; loop de desarrollo más lento.
- **Sin `egui-phosphor`** → embeber la fuente Phosphor (TTF) o migrar a iconos SVG.
- **Fullscreen / monitor secundario en Wayland/Niri** → validar; conservar el
  `niri msg action move-column-to-monitor-next` si sigue siendo necesario.
- **Regresión** si se cambia el binario por defecto antes de la paridad → mitigado por la
  coexistencia.

## Open Questions

- ¿Embeber Phosphor TTF en GPUI o migrar a iconos SVG?
- ¿Mantener `rfd` o usar el file dialog nativo de GPUI?
- ¿El panel de notas puede quedar para una fase posterior sin bloquear el núcleo?
