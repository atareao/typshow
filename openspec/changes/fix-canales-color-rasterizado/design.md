# Design

## Context

`render_page` (`src/gpui_ui/deck.rs`) convierte el pixmap de Typst (RGBA premultiplicado,
vía tiny-skia) al formato interno de GPUI (`RenderImage`, BGRA no premultiplicado)
llamando a `gpui::swap_rgba_pa_to_bgra`.

## Goals / Non-Goals

**Goals:**
- Preservar los colores al rasterizar cualquier página.

**Non-Goals:**
- Cambiar el formato interno de `RenderImage` o el pipeline de render.

## Decisions

- **Aplicar el swap píxel a píxel**: `gpui::swap_rgba_pa_to_bgra` opera sobre **un** píxel
  (4 bytes) — así lo usa GPUI en `svg_renderer.rs`
  (`for pixel in buffer.chunks_exact_mut(4) { swap_rgba_pa_to_bgra(pixel); }`). Pasarle el
  buffer completo convierte solo el primer píxel y deja el resto en RGBA, que GPUI
  interpreta como BGRA → R y B intercambiados. La corrección es iterar con
  `chunks_exact_mut(4)`.

## Risks / Trade-offs

- Coste O(n) por página; despreciable frente al propio rasterizado.
