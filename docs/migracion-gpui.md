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
| `src/gpui_ui/keys.rs` | Teclado vía `on_key_down`: navegación (`apply_nav`) y helpers de frontera de carácter. El antiguo editor del M3 (`apply_edit`/`display_with_caret`) queda como código legacy sin cablear. |
| `src/gpui_ui/edit.rs` | Modelo de edición puro `TextEdit`: cursor, selección con dirección, rango marcado de IME y mapa byte↔UTF-16. |
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
- **Render asíncrono de páginas:** la rasterización Typst→RGBA se ejecuta en
  segundo plano (`cx.background_spawn`), nunca en el hilo de UI. `Deck::image_at`
  devuelve `None` mientras la página se está rasterizando (`is_pending`), la UI
  muestra «Cargando…» y, al terminar, la imagen entra en la caché y se notifica.
- **Caché acotada (LRU):** como máximo `CACHE_CAP` (6) páginas en memoria; al
  desalojar se libera también la textura GPU con `cx.drop_image`. La decisión de
  desalojo (`eviction_candidate`) es una función pura cubierta por tests.
- **Editor de notas/título (M4):** `PresenterView` implementa
  `gpui::EntityInputHandler` sobre un modelo de edición puro (`TextEdit`, en
  `src/gpui_ui/edit.rs`): cursor y selección **con dirección** (ancla + extremo
  activo), rango marcado de composición IME y mapa byte↔UTF-16 (todos los rangos
  que se cruzan con GPUI van en UTF-16). Con la edición activa: `←`/`→`/`Home`/
  `End` mueven el cursor, `Shift+flechas` extienden la selección, el ratón coloca
  el cursor o arrastra para seleccionar (con `Shift` extiende en vez de
  recolocar), `Backspace`/`Delete` borran la selección si la hay y si no el
  carácter contiguo, `Enter` inserta un salto en la nota o guarda el título
  (`save_heading`) y `Esc` cancela sin guardar. El portapapeles está completo:
  `Ctrl+C` copia la selección, `Ctrl+X` la copia y la borra y `Ctrl+V` la pega
  (sin selección, copiar/cortar son no-op); en el título el pegado se aplana a una
  sola línea. La selección se resalta y el cursor (`|`) se pinta solo cuando la
  selección está colapsada, todo desde `editor_frame`/`EditorLayout`. Escribir en
  el **cuerpo** de la nota marca `dirty` (autoguardado); el **título** nunca marca
  `dirty` porque se persiste al pulsar `Enter`. Los caracteres imprimibles no los
  inserta el handler de teclas: los entrega la plataforma vía
  `replace_text_in_range` con el `key_char`.

  > Nota (M4): de `src/gpui_ui/keys.rs` solo siguen cableados `apply_nav`
  > (navegación de páginas, presentador y audiencia) y los helpers de frontera de
  > carácter (`prev_boundary`/`next_boundary`/`clamp_boundary`, que usa
  > `TextEdit`). El editor del M3 (`apply_edit`, `insert_at`, `backspace_at`,
  > `delete_at`, `display_with_caret`) es **código legacy ya sin cablear**
  > (ningún llamador fuera del propio módulo) y es candidato a borrarse al cerrar
  > M4; de momento se conserva.

## Carga asíncrona de documentos (M3)

Abrir un `.typ` desde la UI no bloquea el hilo de UI:

- **`Deck::open_file_async`** marca `loading`, limpia el error previo y lanza en
  `cx.background_spawn` la lectura del fichero, la compilación Typst y la
  apertura de la base de datos de notas (`Notes::with_db`, SQLite + migración).
  Al terminar aplica el resultado en el hilo de UI con `apply_compiled` y
  notifica. `compile_typst` va envuelto en `catch_unwind`, así que un pánico del
  compilador se convierte en `Err("Error interno compilando Typst")` y `loading`
  nunca queda colgado.
- **`doc_generation`** es un contador que se incrementa en cada carga
  (`begin_load`). Cada compilación y cada rasterización captura su generación y
  descarta su resultado si ya no es la vigente: gana la **última apertura
  pedida**, no la que termine más tarde, y las páginas del documento viejo no
  contaminan el nuevo.
- **`image_at` devuelve `None` mientras `loading`**: el documento anterior está a
  punto de ser reemplazado, así que rasterizarlo sería trabajo tirado. La UI
  muestra «Compilando…» en presenter y audiencia. El guard de generación cubre
  además la carrera de un render ya lanzado antes de empezar la carga.
- **`rfd::AsyncFileDialog`**: el diálogo se espera dentro de
  `cx.background_spawn` (sin hilo ni canal propios). El flag `dialog_open` evita
  reentradas y deshabilita visualmente el botón «Abrir» mientras hay un diálogo
  abierto o una compilación en curso.

**Limitación residual:** la ruta síncrona de arranque (`Deck::open_file`, usada
por el CLI) llama a `clear_cache(None)`, que vacía la caché sin liberar texturas
porque no tiene `Context`. Solo es segura con la caché vacía, que es el caso al
arrancar; si se reutilizara en caliente habría que pasarle un `Context` para
hacer `cx.drop_image`.

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

- ~~Editor de notas con `EntityInputHandler` para selección con ratón/teclado,
  resaltado de selección y *pre-edit* de IME~~ ✅ hecho en M4 (ver «Cómo funciona»).
- Borrar el código legacy del editor en `src/gpui_ui/keys.rs` (`apply_edit`,
  `insert_at`, `backspace_at`, `delete_at`, `display_with_caret`) al cerrar M4.
- Límite de tamaño de la caché de páginas (`window.drop_image`).
- Tema Adwaita/GTK4 e iconos (fase 5).
- Retirada de `egui` (fase 6) y cierre (fase 7).
