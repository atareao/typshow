# Proposal

## Why

Hoy el tema (claro/oscuro) no se guarda: cada vez que se abre la app se arranca en claro,
aunque el usuario haya elegido oscuro. Esto obliga a cambiar el tema en cada sesión.

## What Changes

- Persistir la preferencia de tema del usuario entre sesiones en un fichero de configuración.
- Cargar esa preferencia al arrancar (claro/oscuro).
- Guardarla cada vez que el usuario alterna el tema (botón "Tema" o tecla `t`).
- Ubicación: directorio de configuración XDG (`$XDG_CONFIG_HOME/typshow/theme`, con
  `$HOME/.config/typshow/theme` como respaldo). Contenido: `dark` o `light`.

## Capabilities

### New Capabilities

<!-- Ninguna -->

### Modified Capabilities

- `gpui-ui`: el requirement "Tema claro/oscuro estilo Adwaita" pasa a exigir que la app
  recuerde la preferencia de tema entre sesiones.

## Impact

- Nuevo módulo `src/gpui_ui/config.rs` (lectura/escritura de la preferencia de tema).
- `src/gpui_ui/deck.rs` (`Deck`: campo de ruta de configuración, carga al abrir y guardado al alternar).
- `src/gpui_ui/mod.rs` (registro del módulo).
- Sin nuevas dependencias (se usa `std` + variables de entorno XDG).
- Sin cambios de API pública relevante.
