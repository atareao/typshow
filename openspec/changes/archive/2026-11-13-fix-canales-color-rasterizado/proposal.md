# Proposal

## Why

Al rasterizar una página Typst, los colores se intercambian entre rojo y azul (R↔B): el
amarillo se ve cian. Afecta a cualquier color no neutro del documento (p. ej. el logo de
una presentación), mientras que el texto negro/blanco no lo delata.

## What Changes

- Corregir `rgba_to_bgra` en `src/gpui_ui/deck.rs` para que convierta **todos** los
  píxeles, no solo el primero.
- Cubrir el caso con tests que usen buffers de varios píxeles (el test anterior probaba un
  único píxel y enmascaraba el fallo).

## Capabilities

### New Capabilities

<!-- Ninguna -->

### Modified Capabilities

- `gpui-ui`: el requirement "Render de páginas Typst" pasa a exigir que el rasterizado
  preserve los colores (canales correctos en cada píxel).

## Impact

- `src/gpui_ui/deck.rs` (`render_page`, `rgba_to_bgra`).
- Sin cambios de API pública ni de dependencias.
