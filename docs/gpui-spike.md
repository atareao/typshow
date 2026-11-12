# Spike: migración de la UI a GPUI

- **Issue:** [#12 — Migrar la interfaz a GPUI](https://github.com/atareao/typshow/issues/12)
- **Change OpenSpec:** `openspec/changes/gpui-spike/`
- **Plan:** `plans/PLAN-001.md`
- **Rama:** `feature/gpui-spike`
- **Artefacto:** `src/bin/gpui_spike.rs` (binario aislado; la app egui **no** se ha tocado)

## Qué es

Prueba de concepto que sustituye `egui`/`eframe` por **GPUI** (el framework de Zed) en un
binario separado, para evaluar la viabilidad de migrar Typshow. Abre dos ventanas nativas
(presenter + audiencia a pantalla completa) que comparten estado mediante una `Entity` de
GPUI y renderizan la página actual de un documento Typst como imagen.

## Ficheros implicados

| Fichero | Cambio |
|---|---|
| `Cargo.toml` | + `gpui` y `gpui_platform` (git desde `zed-industries/zed`, rev `a1f2e21c`), + `image = "0.25"` |
| `Cargo.lock` | actualizado (`cargo update`) para resolver conflictos de Zed |
| `src/bin/gpui_spike.rs` | binario del prototipo (nuevo) |
| `src/render.rs` | **sin cambios** (se reutiliza `typshow::TypstWorld` desde el binario) |

## Cómo funciona

1. `main` lee la ruta del `.typ` de `argv[1]`, crea una `Entity<Deck>` con
   `cx.new(...)` y abre dos ventanas con `cx.open_window(...)`:
   - presenter: `WindowBounds::Windowed(...)`; muestra **dos paneles apilados**:
     *Actual* (página en curso) y *Siguiente* (previsualización de la página siguiente,
     separada por un borde; en la última página muestra «Fin de la presentacion»),
     cada uno con `padding` (`p_8`) alrededor de la hoja;
   - audiencia: `WindowBounds::Fullscreen(Bounds::default())`; muestra la página
     actual a pantalla completa (`img(...).size_full().object_fit(Contain)`), sin rótulos.
2. `Deck` guarda el `PagedDocument` compilado, la página actual, el tema y una caché
   `HashMap<usize, Arc<RenderImage>>`.
3. Cada vista se suscribe a la entidad con `cx.observe(&deck, |_,_,cx| cx.notify())`,
   de modo que cualquier cambio de página repinta **ambas** ventanas.
4. Render de página: se reutiliza la lógica de `src/render.rs::get_page` →
   `typst_render::render(...)` devuelve un `tiny_skia::Pixmap` con RGBA **premultiplicado**;
   `rgba_to_bgra` lo pasa a BGRA **no premultiplicado** (swap R/B + división por alpha, igual
   que `gpui::swap_rgba_pa_to_bgra`) y se envuelve en `RenderImage::new(vec![image::Frame::new(rgba)])`;
   se muestra con `img(arc).object_fit(ObjectFit::Contain)`.
5. Navegación: botones (`.on_mouse_down`) y atajos de teclado (`actions!` + `bind_keys` +
   `.on_action` con `track_focus`):
   - estándar: `→`/espacio (siguiente), `←` (anterior), `Home` (inicio), `End` (final), `t` (tema);
   - estilo vim: `l`/`j` (siguiente), `h`/`k` (anterior), `g` (inicio), `G`/`shift-g` (final).
6. Errores: si no hay fichero o falla la compilación Typst, se muestra el mensaje en vez
   de una página en blanco.

## Variables / configuración

- Sin variables de entorno propias. Se reutiliza la búsqueda de paquetes Typst de
  `TypstWorld` (`TYPST_PACKAGES_DIR`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`).
- Ejecución: `cargo run --bin gpui_spike -- <fichero.typ>`

## Verificación realizada

- `cargo build --bin gpui_spike` → **Finished** (sin errores).
- `cargo test --bin gpui_spike` → `rgba_to_bgra_swaps_red_and_blue` **ok**.
- `cargo build` y `cargo test` completos → **verdes** (61 + 1 + 2 = 64 tests; la app egui intacta).
- Arranque en Wayland/Niri: `timeout 10s cargo run --bin gpui_spike -- demo.typ` → **exit 124**
  (la app se mantuvo viva sin panic; no se pudo confirmar visualmente el contenido).

## Hallazgos

### A favor
- **Multi-ventana funciona**: `cx.open_window` abre ventanas nativas independientes y
  `WindowBounds::Fullscreen` da pantalla completa. El estado compartido vía `Entity` +
  `observe` sincroniza ambas ventanas sin `Arc<Mutex>`.
- **Render de imagen resuelto**: el `Pixmap` de Typst se convierte a `RenderImage` con un
  swap RGBA→BGRA más des-premultiplicación de alpha. No hizo falta tocar `render.rs`.
- **API más limpia para estado**: el modelo `Entity`/`Context` es más estructurado que el
  `Arc<Mutex<AppState>>` actual.

### En contra / riesgos
- **GPUI NO está publicado oficialmente** en crates.io. Hay builds no oficiales
  (`gpui-unofficial`, `gpui-x`, `fc-gpui`, `gpui-fast`) y la vía git desde Zed. Se eligió
  git con `rev` fijado → reproducible pero **compila buena parte del árbol de Zed**
  (clone de ~400 MB, build muy largo la primera vez).
- **Conflictos de `Cargo.lock`**: dependencias transitivas de Zed (incluidas las gated a
  wasm/ios, que igual entran en el lock) chocaban con las de Typst
  (`unicode-properties = "=0.1.3"`, `futures-util`). Resuelto con `cargo update` amplio.
- **Framework pre-1.0** con breaking changes frecuentes; el `rev` habrá que subirlo a mano.
- **Diferencias de API notables** frente a egui: no es modo inmediato puro; `on_click` solo
  existe en elementos *stateful* (requiere `.id()`), por eso se usó `on_mouse_down`;
  el teclado exige gestión de foco (`FocusHandle` + `actions!`).
- **Pérdida de ecosistema**: no hay equivalente directo a `egui-phosphor` ni a los overrides
  Adwaita de `theme.rs`; habría que reconstruir el theme (existe ecosistema `gpui-component`).
- **Fuentes**: no se ha podido confirmar visualmente el render de texto de la UI (solo se
  verificó el arranque sin panic).

## Veredicto

**Viabilidad técnica: SÍ.** El spike demuestra que GPUI cubre los tres pilares del caso de
uso (doble ventana, fullscreen y render de páginas Typst como imagen) y compila, arranca y
pasa tests en el entorno objetivo (Wayland/Niri).

**Recomendación: NO migrar todavía.** El coste/riesgo actual es alto: dependencia no oficial
y pre-1.0, árbol de compilación enorme y pérdida del ecosistema egui. Se recomienda:

1. Mantener egui como UI principal.
2. Revaluar GPUI cuando se publique oficialmente y estabilice su API.
3. Si se decide migrar, hacerlo por fases (primero el presenter, luego la audiencia) reusando
   este prototipo como base.

> Este veredicto es una recomendación técnica; la decisión final corresponde al responsable
> del proyecto.

## Follow-ups

- Confirmar visualmente (render de texto y de página) en una sesión gráfica.
- Evaluar `gpui-component` para recuperar el look Adwaita.
- Decidir si se archiva el change `gpui-spike` o se convierte en plan de migración.
