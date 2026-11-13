# Fix: canales de color al rasterizar (amarillo → cian)

## Qué es

Corrección del bug por el que una página Typst rasterizada mostraba los colores con rojo y azul
intercambiados (R↔B): cualquier color no neutro salía mal (el amarillo `#ffff00` se veía cian),
mientras que el texto negro/blanco no lo delataba porque R=G=B.

## Ficheros implicados

- `src/gpui_ui/deck.rs` — `render_page` y `rgba_to_bgra`.

## Cómo funciona

`render_page` rasteriza la página con `typst_render` (pixmap RGBA premultiplicado) y lo convierte
al formato interno de GPUI (`RenderImage`, **BGRA no premultiplicado**) con `rgba_to_bgra`.

La clave: `gpui::swap_rgba_pa_to_bgra` opera sobre **un solo píxel** (4 bytes) — así lo usa GPUI en
su propio `svg_renderer.rs` (`for pixel in buffer.chunks_exact_mut(4) { swap_rgba_pa_to_bgra(pixel) }`).
Antes se le pasaba el buffer completo, de modo que **solo se convertía el primer píxel**.

La corrección itera por píxel:

```rust
for pixel in data.as_chunks_mut::<4>().0 {
    gpui::swap_rgba_pa_to_bgra(pixel);
}
```

`as_chunks_mut::<4>()` en lugar de `chunks_exact_mut(4)` porque clippy 1.100 marca
`clippy::chunks_exact_to_as_chunks` (el CI usa una toolchain más nueva que la local).

## Variables / configuración

Ninguna.

## Cómo habilitar / desplegar

Es un fix de código; entra con el binario. No requiere migración ni configuración.

## Decisiones

- Delegar en `gpui::swap_rgba_pa_to_bgra` (no reimplementar) para no divergir del formato interno de GPUI.
- Aplicar el swap por píxel, replicando el uso canónico de GPUI.

## Follow-ups

- Verificación visual en Niri del resto de elementos (tema claro/oscuro, los 17 iconos, markdown de las notas, `Esc` cierra la proyección).
