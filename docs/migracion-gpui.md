# Migración de la UI a GPUI

## Qué es

Sustitución progresiva de la UI de Typshow, escrita con `egui`/`eframe`, por una
UI nativa basada en **GPUI** (el framework de Zed). Durante la migración conviven
dos binarios:

- `typshow` — UI `egui` original (intacta).
- `typshow-gpui` — UI nueva en GPUI.

Cuando la paridad funcional esté completa, el binario GPUI pasará a llamarse
`typshow` y se retirará `egui`.

## Ficheros implicados

| Fichero | Responsabilidad |
|---|---|
| `src/gpui_ui/mod.rs` | Módulo raíz de la UI GPUI. |
| `src/gpui_ui/deck.rs` | Estado compartido (`Deck`): navegación, tema, fichero, notas y caché de páginas renderizadas. |
| `src/gpui_ui/keys.rs` | Manejo de teclado (navegación vim + edición de notas/título) vía `on_key_down`. |
| `src/gpui_ui/presenter.rs` | Ventana del presentador: paneles *Actual*/*Siguiente*, toolbar, notas y gestión de la ventana de audiencia. |
| `src/gpui_ui/audience.rs` | Ventana de audiencia (pantalla completa). |
| `src/gpui_ui/theme.rs` | Paleta claro/oscuro. |
| `src/bin/typshow_gpui.rs` | Punto de entrada del binario GPUI. |

Se reutiliza el dominio sin cambios: `TypstWorld`, `Notes`, `NotesDb` y el
pipeline Typst→`RenderImage`.

## Cómo funciona

- **Estado compartido:** una única `Entity<Deck>` (GPUI) es observada por ambas
  ventanas. Cualquier cambio hace `cx.notify()` y las vistas se repintan.
- **Render:** las páginas Typst se rasterizan a `RenderImage` y se cachean en
  `Deck.cache` (`HashMap<usize, Arc<RenderImage>>`).
- **Teclado:** se gestiona con `on_key_down` en la raíz de cada ventana (no con
  `KeyBinding`), porque GPUI despacha los keybindings *antes* que `on_key_down` y
  un atajo global secuestraría la escritura en el editor de notas. Atajos:
  `→/espacio/l/j` siguiente, `←/h/k` anterior, `g`/`G` inicio/fin, `t` tema.
- **Notas:** panel lateral con ver/editar/guardar/cancelar/nueva/eliminar/
  reimportar y edición del título (`Título`).

## Ventanas y `app_id`

Cada ventana publica un `app_id` estable para poder usar reglas de compositor:

- Presentador: `com.atareao.typshow.presenter`
- Audiencia: `com.atareao.typshow.audience`

## Audiencia en el monitor secundario (Niri)

GPUI solo honra `display_id` en el backend **X11**; en **Wayland** (Niri) los
toplevel lo ignoran, así que la ventana de audiencia se abre en la salida activa.
La forma nativa de fijarla es una **regla de ventana** en Niri:

```kdl
// ~/.config/niri/config.kdl
window-rule {
    match app-id="com.atareao.typshow.audience"
    open-on-output "DP-1"      // nombre de la salida del proyector (niri msg outputs)
    open-fullscreen true
}
```

El nombre de la salida se obtiene con `niri msg outputs` (p. ej. `HDMI-A-1`,
`DP-1`). Tras editar la config, recargar con `niri msg action reload-config`.

## Decisiones

- **Coexistencia con dos binarios** en vez de un big-bang: permite verificar la
  paridad fase a fase sin romper la app egui.
- **`Entity<Deck>`** como estado compartido (en lugar de `Arc<Mutex>`) para
  integrarse con el modelo de observación de GPUI.
- **Teclado por `on_key_down`** para evitar el conflicto keybinding/edición.

## Pendiente / follow-ups

- Editor de notas con `EntityInputHandler` (soporte de acentos/IME).
- `rfd` asíncrono y compilación Typst fuera del hilo de UI.
- Límite de tamaño de la caché de páginas (`window.drop_image`).
- Tema Adwaita/GTK4 e iconos (fase 5).
- Retirada de `egui` (fase 6) y cierre (fase 7).
