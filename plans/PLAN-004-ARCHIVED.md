# PLAN-004 — Fix: colores invertidos al rasterizar (amarillo → cian)

- **Issue:** #28
- **PRs:** #27 (fix de código), #29 (archivado del change OpenSpec)
- **Estado:** completado y archivado

## Objetivo

Corregir la inversión de canales R↔B al rasterizar páginas Typst: el amarillo se veía cian.

## Causa raíz

`src/gpui_ui/deck.rs` (`render_page`) llamaba `gpui::swap_rgba_pa_to_bgra` con el **buffer entero**,
pero esa función de GPUI opera sobre **un píxel** (4 bytes): solo se convertía el primer píxel y el
resto quedaba RGBA leído como BGRA → R y B intercambiados.

## Tareas

- [x] 1. Test RED: `rgba_to_bgra_converts_every_pixel` (buffer de 2 píxeles) y `render_page_preserves_colors` (página amarilla, todos los píxeles BGRA).
- [x] 2. GREEN: iterar por píxel con `as_chunks_mut::<4>()` (mismo patrón que el `svg_renderer` de GPUI).
- [x] 3. REFACTOR: `cargo fmt` + clippy 1.100.0 (`as_chunks_mut` por el lint `chunks_exact_to_as_chunks`).
- [x] 4. Auditoría `@rust-reviewer`: aprobada.
- [x] 5. Verificación visual en Niri: el logo (Tux) muestra el pico/patas amarillos correctos.
- [x] 6. Change OpenSpec `fix-canales-color-rasterizado` archivado; delta aplicado a `openspec/specs/gpui-ui/spec.md`.
