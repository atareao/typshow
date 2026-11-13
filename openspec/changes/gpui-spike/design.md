# Design

## Context

Ver `proposal.md` - Why. La app actual usa egui/eframe con `ctx.show_viewport_immediate` para las dos ventanas y `SharedState = Arc<Mutex<AppState>>`. El render de Typst produce un `tiny_skia::Pixmap` RGBA que hoy se sube a textura egui con `ctx.load_texture`.

## Goals / Non-Goals

**Goals:**
- Validar en GPUI: multi-ventana, fullscreen, estado compartido, render de imagen e input.
- Aislar el prototipo del código egui existente.

**Non-Goals:**
- Paridad visual Adwaita.
- Edición de notas, empaquetado y release.

## Decisions

- **Dependencia git desde Zed** (frente a crates no oficiales de crates.io): fuente canónica y actualizada; se fija un `rev` concreto para reproducibilidad. Contrapartida: compila buena parte del árbol de Zed y puede romperse al actualizar.
- **Binario separado** `src/bin/gpui_spike.rs` (frente a feature flag): evita tocar `main.rs` y mantiene intacto el build egui; permite comparar ambos.
- **Estado compartido con `Entity`** (frente a `Arc<Mutex>`): patrón idiomático de GPUI; cada ventana observa la entidad y repinta.
- **Render a `RenderImage`**: `typst_render::render` → `tiny_skia::Pixmap` RGBA → swap de canales a BGRA → `RenderImage::new(vec![Frame::new(RgbaImage)])`, mostrado con `img()`.
- **Audiencia fullscreen** con `WindowBounds::Fullscreen`.

## Risks / Trade-offs

- GPUI pre-1.0 con breaking changes → pin de `rev` en la dependencia git.
- Compilación pesada del árbol de Zed → asumido por tratarse de un spike.
- Fullscreen y selección de monitor en Wayland/Niri → validar manualmente; si falla, documentarlo como limitación en el informe.
- Sin equivalente a `egui-phosphor` → usar texto/iconos unicode simples en el prototipo.

## Open Questions

- ¿GPUI permite abrir la ventana fullscreen en un monitor concreto de forma estable? Se responde durante el spike.