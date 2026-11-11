# Tasks

## 1. Preparación

- [x] 1.1 Crear branch `feature/gpui-spike` desde `development` y verificar con `git status` que el árbol queda limpio
- [x] 1.2 Añadir dependencias `gpui`/`gpui_platform` (git, `rev` fijado) al Cargo.toml y verificar que `cargo metadata` resuelve sin errores

## 2. Prototipo de ventanas

- [x] 2.1 Crear `src/bin/gpui_spike.rs` con una ventana GPUI mínima y verificar `cargo build --bin gpui_spike`
- [x] 2.2 Abrir la segunda ventana (audiencia) con `WindowBounds::Fullscreen` y verificar que ambas se abren en Niri/Wayland
- [x] 2.3 Implementar el estado compartido como `Entity` y verificar que un cambio se propaga a ambas ventanas

## 3. Render de páginas

- [x] 3.1 Reutilizar `TypstWorld`/`Renderer` para compilar el documento y verificar que `cargo test` sigue en verde
- [x] 3.2 Convertir el pixmap RGBA a `RenderImage` (swap a BGRA) y mostrar la página con `img()`; verificar visualmente
- [x] 3.3 Manejar el error de compilación mostrando un mensaje; verificar con un fichero .typ inválido

## 4. Navegación y tema

- [x] 4.1 Añadir botones Start/Prev/Next/End y atajos de teclado; verificar los límites de página
- [x] 4.2 Sincronizar el repintado de la audiencia al cambiar de página; verificar en ejecución
- [x] 4.3 Añadir toggle de tema claro/oscuro; verificar coherencia en ambas ventanas

## 5. Cierre

- [x] 5.1 Escribir `docs/gpui-spike.md` con hallazgos y veredicto; verificar que existe y es coherente
- [x] 5.2 Verificar `cargo build` y `cargo test` completos en verde y actualizar `plans/PLAN-001.md`