# Design

## Context

El tema vive en `Deck.dark: bool` (`src/gpui_ui/deck.rs`), que arranca en `false` y se
alterna con `Deck::toggle_theme` (llamado desde el botón "Tema" y desde la tecla `t` vía
`apply_nav`). No existe ninguna persistencia.

## Goals / Non-Goals

**Goals:**
- Recordar la preferencia de tema entre sesiones sin añadir dependencias.

**Non-Goals:**
- Persistir otras preferencias (tamaño de ventana, panel de notas, etc.).

## Decisions

- **Fichero de configuración XDG**: `$XDG_CONFIG_HOME/typshow/theme` con respaldo
  `$HOME/.config/typshow/theme`. Contenido mínimo: `dark` o `light` (una línea).
  Se evita añadir `dirs`/`serde`/`toml` para mantener el binario ligero.
- **Módulo `config`** (`src/gpui_ui/config.rs`) con funciones puras y testeables que
  reciben la ruta como parámetro (`load_theme_from`, `save_theme_to`) y `theme_path` para
  resolver la ruta por entorno.
- **Inyección de la ruta en `Deck`**: `Deck` guarda `theme_path: Option<PathBuf>`. Los tests
  pasan una ruta temporal (sin tocar el entorno); `Deck::empty()` lo deja en `None` (sin
  persistencia), de modo que los tests existentes no escriben en disco.
- **Punto de carga**: `Deck::load` fija `theme_path` y carga el tema. `Deck::toggle_theme`
  alterna y guarda (best-effort; un fallo de escritura solo se registra, no rompe la UI).

## Risks / Trade-offs

- Escritura best-effort: si el directorio de configuración no es escribible, el tema no
  persiste pero la app sigue funcionando (solo se registra el error).
