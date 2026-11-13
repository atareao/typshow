# Tasks

## 1. Preparación y estructura

- [x] 1.1 Crear branch `feature/gpui-migration` desde `development` (o desde el spike) y verificar `git status`
- [x] 1.2 Crear la estructura `src/gpui/` (app, presenter, audience, theme, icons) y un binario `typshow-gpui` coexistiendo con egui; verificar `cargo build`
- [x] 1.3 Definir la estrategia de coexistencia en `Cargo.toml` (dos binarios) y verificar que `typshow` (egui) sigue funcionando

## 2. Núcleo: estado y render

- [x] 2.1 Implementar `Deck` con `TypstWorld` y caché de páginas `HashMap<usize, Arc<RenderImage>>`; verificar con test
- [x] 2.2 Reutilizar `AppState` (navegación), `Notes` y `NotesDb` sin dependencias de egui; verificar `cargo test`
- [x] 2.3 Manejar errores de carga/compilación mostrando un mensaje; verificar con un `.typ` inválido
- [x] 2.4 Rasterizar las páginas fuera del hilo de UI (tarea en segundo plano) y acotar la caché a una LRU (`CACHE_CAP`) liberando la textura al desalojar; verificar con tests y en ejecución
- [x] 2.5 Compilar Typst en segundo plano al abrir un `.typ` desde la UI (`open_file_async`), mostrando estado de carga y aplicando el resultado en el hilo de UI; verificar con tests

## 3. Ventana de presentador

- [x] 3.1 Reproducir los paneles *Actual* y *Siguiente* con padding; verificar visualmente
- [x] 3.2 Implementar la barra de navegación (Inicio/Anterior/Siguiente/Final) con estado disabled; verificar límites
- [x] 3.3 Añadir "Abrir fichero" (`rfd`) filtrando `.typ`; verificar carga
- [x] 3.4 Añadir el toggle de proyección (mostrar/ocultar audiencia); verificar
- [x] 3.5 Añadir el toggle de tema claro/oscuro; verificar coherencia
- [x] 3.6 Implementar el panel de notas (ver/editar/guardar/cancelar/nueva/eliminar/reimportar); verificar persistencia
- [x] 3.8 Editar el título (heading) de la nota desde el panel
- [x] 3.7 Implementar atajos de teclado y navegación por clic; verificar
- [x] 3.9 Editor de notas con cursor: inserción en la posición del cursor, flechas/Home/End, Backspace/Delete y pegar (Ctrl+V); verificar con tests
- [x] 3.10 Abrir el diálogo de fichero (`rfd`) sin bloquear el hilo de UI; verificar que la ventana sigue respondiendo (verificación manual del diálogo)
- [x] 3.11 Modelo de edición puro `TextEdit` (cursor, selección con dirección, rango marcado de IME, mapa byte↔UTF-16) con tests
- [x] 3.12 `EntityInputHandler` en la ventana de presentador para notas y título: selección con Shift+flechas y ratón, resaltado, pre-edit de IME y portapapeles
  - [x] (a) handler, (b) render, (c) ratón, (e) teclado y (f) tests — hechos y auditados
  - [x] (d) portapapeles: copiar (Ctrl+C) y cortar (Ctrl+X) en nota y título, con pegado (Ctrl+V)

## 4. Ventana de audiencia

- [x] 4.1 Implementar la ventana audiencia con `WindowBounds::Fullscreen`; verificar en Niri/Wayland
- [x] 4.2 Sincronizar página y tema con el presentador; verificar en ejecución
- [x] 4.3 Implementar atajos y clic en la audiencia, y cierre con Esc; verificar
- [x] 4.4 Audiencia en monitor secundario: GPUI ignora `display_id` en toplevel Wayland; se fija `app_id` (`com.atareao.typshow.audience`) y se documenta la regla de compositor (Niri `open-on-output`)

## 5. Tema e iconos

- [x] 5.1 Reescribir el estilo Adwaita/GTK4 en GPUI (colores, radios, spacing); verificar visualmente
- [x] 5.2 Sustituir `egui-phosphor` por iconos en GPUI (Phosphor embebido o SVG); verificar que se ven (verificación visual en Niri pendiente)

## 6. Cambio a framework por defecto

- [ ] 6.1 Renombrar el binario GPUI a `typshow` y retirar la app egui; verificar `cargo build`
- [ ] 6.2 Retirar `egui`/`eframe`/`egui-phosphor` de `Cargo.toml`; verificar `cargo build` y `cargo test`
- [ ] 6.3 Verificar paridad funcional completa frente a la app egui

## 7. Cierre

- [ ] 7.1 Escribir `docs/migracion-gpui.md` con el resultado y las decisiones
- [ ] 7.2 Verificar `cargo build`, `cargo test` y `cargo clippy` en verde; actualizar `plans/PLAN-002.md`
