# Tasks

## 1. Preparación y estructura

- [x] 1.1 Crear branch `feature/gpui-migration` desde `development` (o desde el spike) y verificar `git status`
- [x] 1.2 Crear la estructura `src/gpui/` (app, presenter, audience, theme, icons) y un binario `typshow-gpui` coexistiendo con egui; verificar `cargo build`
- [x] 1.3 Definir la estrategia de coexistencia en `Cargo.toml` (dos binarios) y verificar que `typshow` (egui) sigue funcionando

## 2. Núcleo: estado y render

- [x] 2.1 Implementar `Deck` con `TypstWorld` y caché de páginas `HashMap<usize, Arc<RenderImage>>`; verificar con test
- [ ] 2.2 Reutilizar `AppState` (navegación), `Notes` y `NotesDb` sin dependencias de egui; verificar `cargo test`
- [x] 2.3 Manejar errores de carga/compilación mostrando un mensaje; verificar con un `.typ` inválido

## 3. Ventana de presentador

- [x] 3.1 Reproducir los paneles *Actual* y *Siguiente* con padding; verificar visualmente
- [x] 3.2 Implementar la barra de navegación (Inicio/Anterior/Siguiente/Final) con estado disabled; verificar límites
- [x] 3.3 Añadir "Abrir fichero" (`rfd`) filtrando `.typ`; verificar carga
- [x] 3.4 Añadir el toggle de proyección (mostrar/ocultar audiencia); verificar
- [x] 3.5 Añadir el toggle de tema claro/oscuro; verificar coherencia
- [x] 3.6 Implementar el panel de notas (ver/editar/guardar/cancelar/nueva/eliminar/reimportar); verificar persistencia
- [ ] 3.8 Editar el título (heading) de la nota desde el panel
- [x] 3.7 Implementar atajos de teclado y navegación por clic; verificar

## 4. Ventana de audiencia

- [x] 4.1 Implementar la ventana audiencia con `WindowBounds::Fullscreen`; verificar en Niri/Wayland
- [x] 4.2 Sincronizar página y tema con el presentador; verificar en ejecución
- [x] 4.3 Implementar atajos y clic en la audiencia, y cierre con Esc; verificar
- [ ] 4.4 Mover la audiencia al monitor secundario (Niri) si aplica; verificar o documentar limitación

## 5. Tema e iconos

- [ ] 5.1 Reescribir el estilo Adwaita/GTK4 en GPUI (colores, radios, spacing); verificar visualmente
- [ ] 5.2 Sustituir `egui-phosphor` por iconos en GPUI (Phosphor embebido o SVG); verificar que se ven

## 6. Cambio a framework por defecto

- [ ] 6.1 Renombrar el binario GPUI a `typshow` y retirar la app egui; verificar `cargo build`
- [ ] 6.2 Retirar `egui`/`eframe`/`egui-phosphor` de `Cargo.toml`; verificar `cargo build` y `cargo test`
- [ ] 6.3 Verificar paridad funcional completa frente a la app egui

## 7. Cierre

- [ ] 7.1 Escribir `docs/migracion-gpui.md` con el resultado y las decisiones
- [ ] 7.2 Verificar `cargo build`, `cargo test` y `cargo clippy` en verde; actualizar `plans/PLAN-002.md`
