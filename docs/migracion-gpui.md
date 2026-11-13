# Migración de la UI a GPUI

## Qué es

Typshow es un visor de presentaciones Typst con **UI nativa en GPUI** (el framework
de Zed). La migración desde `egui`/`eframe` está **terminada**: `typshow` es el único
binario y `egui`, `eframe`, `egui-phosphor` y `winit` ya no están en `Cargo.toml`.

La app abre dos ventanas que comparten estado:

- **Presentador** — página actual, previsualización de la siguiente, barra de
  navegación, toggle de tema y de proyección, y panel de notas editable.
- **Audiencia** — la página actual a pantalla completa, sin decoraciones; se abre y
  se cierra desde el presentador.

Las notas del presentador son Markdown y se guardan en una base de datos SQLite
junto al `.typ`.

## Ficheros implicados

| Fichero | Responsabilidad |
|---|---|
| `src/main.rs` | Punto de entrada: registra la fuente Phosphor, inicializa `tracing` y abre la ventana del presentador. |
| `src/gpui_ui/presenter.rs` | Ventana del presentador: paneles *Actual*/*Siguiente*, toolbar, notas (ver/editar), título, proyección y `EntityInputHandler`. |
| `src/gpui_ui/audience.rs` | Ventana de audiencia a pantalla completa. |
| `src/gpui_ui/deck.rs` | Estado compartido `Deck`: navegación, tema, fichero, notas, proyección y caché LRU de páginas renderizadas. |
| `src/gpui_ui/edit.rs` | Modelo de edición puro `TextEdit`: cursor, selección con dirección, rango marcado de IME y mapa byte↔UTF-16. |
| `src/gpui_ui/keys.rs` | Navegación por teclado (`apply_nav`) y helpers de frontera de carácter. |
| `src/gpui_ui/icons.rs` | Iconos Phosphor embebidos y helpers `label`/`icon`. |
| `src/gpui_ui/theme.rs` | Tokens de la paleta claro/oscuro (Adwaita/GTK4) y constantes de geometría. |
| `src/notes.rs` | Modelo de notas y parseo de Markdown (`MdElement`). |
| `src/notes_db.rs` | Persistencia SQLite de notas (WAL, migraciones). |
| `src/render.rs` | `TypstWorld`: compilación Typst (sin la parte egui). |
| `assets/fonts/Phosphor.ttf` | Fuente de iconos vendorizada (licencia en `LICENSE-Phosphor.txt`). |

## Cómo funciona

- **Estado compartido:** una única `Entity<Deck>` es observada por ambas ventanas;
  cualquier cambio hace `cx.notify()` y las dos vistas se repintan.
- **Ventana de audiencia:** la abre y la cierra el presentador. El cierre se difiere
  con `cx.defer` porque `window.remove_window()` dispara `on_window_closed` de forma
  síncrona y ese callback vuelve a actualizar el `PresenterView`; hacerlo dentro del
  update reentraría en la `Entity` prestada (doble *lease*). Si la apertura falla se
  registra el error con `tracing::error!` y `projection` queda en `false`: la UI no se
  marca como proyectando sin ventana.
- **Render asíncrono con caché LRU:** la compilación Typst y la rasterización a
  `RenderImage` corren en `cx.background_spawn`, nunca en el hilo de UI. La caché está
  acotada a `CACHE_CAP` (6) páginas; al desalojar se libera la textura GPU con
  `cx.drop_image`. Un contador de generación descarta los resultados de documentos
  viejos (gana la última apertura pedida).
- **Editor de notas y título:** `PresenterView` implementa `gpui::EntityInputHandler`
  sobre `TextEdit`. Cursor y selección con dirección, rango marcado de IME y todos los
  rangos que cruzan GPUI en UTF-16. Portapapeles completo (copiar/cortar/pegar);
  escribir en el cuerpo marca `dirty` (autoguardado) y el título se persiste con Enter.
- **Tema:** tokens Adwaita/GTK4 en `theme.rs` (paleta claro/oscuro, radios, spacing).
- **Iconos:** Phosphor embebido y pintado con `font_family("Phosphor")`. Es
  imprescindible fijar la familia: GPUI crea un único span con la fuente primaria
  cuando no hay `font_fallbacks`, y cosmic-text devuelve la primera cara de la fontdb
  que tenga el codepoint; como los iconos viven en el área de uso privado (PUA), el
  fallback elegía otra fuente para 10 de los 16 codepoints.

## Variables / configuración

- **`RUST_LOG`** — filtro de `tracing` (`EnvFilter::try_from_default_env`, por defecto
  `info`). Ej.: `RUST_LOG=debug cargo run -- demo.typ`.
- **`app_id` de cada ventana** (para reglas de compositor):
  - Presentador: `com.atareao.typshow.presenter`
  - Audiencia: `com.atareao.typshow.audience`
- **Monitor secundario en Niri:** GPUI solo honra `display_id` en X11; en Wayland los
  toplevel lo ignoran, así que la audiencia se abre en la salida activa. Se fija con
  una regla de ventana:

  ```kdl
  // ~/.config/niri/config.kdl
  window-rule {
      match app-id="com.atareao.typshow.audience"
      open-on-output "DP-1"      // salida del proyector (niri msg outputs)
      open-fullscreen true
  }
  ```

  Recargar con `niri msg action reload-config`.

## Cómo ejecutar / desplegar

```bash
cargo run -- <fichero.typ>     # desarrollo
cargo build --release          # binario optimizado en target/release/typshow
./target/release/typshow --help
```

Sin argumentos la app arranca con un mensaje de uso en el panel de error; `-h`/`--help`
imprime el uso por stdout y sale con código 0.

## Decisiones

- **GPUI en vez de egui:** multi-ventana nativa con `Entity`/`Context` (sin
  `Arc<Mutex>`), render de imágenes integrado y una API de estado más estructurada.
  Coste: dependencia git pre-1.0 y compilación pesada del árbol de Zed (rev fijado).
- **Phosphor embebido en vez de SVG:** los iconos se pintan como glifos de texto, sin
  `AssetSource` ni rasterizado propio, y se ven igual en Wayland/Niri. El TTF se
  vendoriza para no depender de un crate externo.
- **Cierre de la audiencia diferido (`cx.defer`):** evita la reentrada en la `Entity`
  descrita arriba; el handle se descarta de inmediato y la ventana desaparece al final
  del efecto.
- **Desviaciones de contraste aceptadas:** el acento `#3584e4` con texto blanco da
  3.77:1 — cumple AA para texto grande y para componentes de interfaz (WCAG 1.4.11),
  no para texto normal. Está documentado en `theme.rs`.
- **`FONT_FAMILY_MONO = "Adwaita Mono"`:** suposición para GNOME; si la familia no
  existe, GPUI cae al stack del sistema sin romper.

## Pendiente / follow-ups

- **Verificación visual en Niri:** tema, iconos, markdown de las notas y Esc/proyección.
- **Editor:** undo/redo y navegación por palabras (Ctrl+flechas).
- **Panel de notas redimensionable.**
- **`MdElement::EmptyLine` sin productor:** `parse_md` nunca lo emite; decidir si se
  elimina o se genera para líneas en blanco.
