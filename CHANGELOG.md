# Changelog
## [0.3.0] - 2026-11-14

### Bug Fixes

- *(gpui)* Renderiza la cursiva en el markdown de las notas
- *(gpui)* Esc cierra la proyección aunque se esté editando

### Documentation

- *(release)* Documenta la 0.2.1 y actualiza el handoff

### Features

- *(gpui)* Persiste la preferencia de tema entre sesiones

### Miscellaneous Tasks

- *(openspec)* Archiva los changes persistir-tema, markdown-cursiva y esc-cierra-proyeccion
## [0.2.1] - 2026-11-13

### Bug Fixes

- *(gpui)* Convierte todos los píxeles de la página a BGRA

### Documentation

- *(release)* Documenta la 0.2.0, actualiza handoff y archiva PLAN-003
- *(handoff)* Marca cerrado el follow-up del release 0.2.0
- *(openspec)* Change fix-canales-color-rasterizado
- *(openspec)* Archiva fix-canales-color-rasterizado y actualiza el handoff
- Documenta el fix de canales de color y actualiza el handoff

### Miscellaneous Tasks

- *(release)* Permite disparar release.yml y hace no bloqueante el publish a crates.io
- *(release)* Sincroniza Cargo.lock con la versión del bump
- Release v0.2.1

### Refactor

- *(gpui)* Usa as_chunks_mut para el lint de clippy 1.100
## [0.2.0] - 2026-11-13

### Bug Fixes

- *(gpui)* Corrige el ciclo de vida de la ventana de audiencia y muestra errores
- *(gpui)* Limita las imágenes del presenter a su panel

### Documentation

- *(openspec)* Inicializa OpenSpec, plan y documentación del spike GPUI
- *(openspec)* Propuesta y plan de migración de la UI a GPUI
- *(handoff)* Relevo tras la Fase 1 de la migración GPUI
- *(gpui)* Actualiza spec OpenSpec, tasks, plan y handoff
- *(handoff)* Relevo tras cerrar 3.12/M4 y commitear la rama
- *(handoff)* Anota el arreglo de rendimiento del perfil dev
- Actualiza estado de PLAN-002 y handoff (3.12/M4 verificado, PR #14 listo)
- *(handoff)* Anota el arreglo de CI (deps de sistema y clippy) y corrige el estado del PR #13
- *(handoff)* Relevo tras mergear el PR #14 (migración GPUI, hito M4)
- *(openspec)* Marcar 2.2 completada en el change migracion-gpui y PLAN-002
- *(gpui)* Documentar la migración y archivar el change (fase 7)
- *(handoff)* Migración GPUI mergeada en development
- *(handoff)* Relevo al release 0.2.0 y plan PLAN-003

### Features

- *(gpui)* Spike de viabilidad con doble ventana y render Typst
- *(gpui)* Fase 1 de la migración — núcleo, presenter y audiencia en GPUI
- *(gpui)* Fase 3 del presenter — abrir fichero, proyección, notas y navegación
- *(gpui)* Edición del título de la nota y audiencia en monitor secundario
- *(gpui)* Editor de notas con cursor, navegación y pegado
- *(gpui)* Editor de notas con selección, IME y portapapeles (3.11/3.12 M4)
- *(gpui)* Iconos Phosphor embebidos en los controles (tarea 5.2)
- *(gpui)* [**breaking**] GPUI como único binario y retirada de egui (fase 6)

### Miscellaneous Tasks

- Añade las dependencias de sistema de GPUI y caché de Cargo
- *(release)* Bump minor para breaking en la serie 0.x y arregla la detección de feat
- Release v0.2.0

### Performance

- *(gpui)* Rasteriza las páginas en segundo plano y acota la caché (LRU)
- *(build)* Optimiza las dependencias a opt-level 3 en el perfil dev

### Refactor

- *(gpui)* Extrae la edición de notas/título a funciones puras + tests de acentos
- *(gpui)* Elimina el editor legacy sin cablear de keys.rs
- Elimina los 22 warnings de clippy (cero warnings)
- *(gpui-spike)* Usa as_chunks_mut para el lint nuevo de clippy
- *(gpui)* Desacoplar Notes de egui moviendo el pintado a notes_egui

### Styling

- *(gpui)* Tema Adwaita/GTK4 con tokens únicos (tarea 5.1)
## [0.1.11] - 2026-07-30

### Miscellaneous Tasks

- Auto-trigger release workflow from release-prepare
- Release v0.1.11
## [0.1.10] - 2026-07-30

### Miscellaneous Tasks

- Add workflow_dispatch trigger to release.yml
- Release v0.1.10
## [0.1.9] - 2026-07-30

### Bug Fixes

- Replace expired GH_PAT with GITHUB_TOKEN in release-prepare

### Features

- *(ui)* Add notes panel visibility toggle (#10)

### Miscellaneous Tasks

- Release v0.1.9
## [0.1.8] - 2026-06-23

### Bug Fixes

- Add crates.io required metadata and fix dead_code warning

### Documentation

- Update README with SQLite notes, emoji, controls detail

### Features

- *(ui)* Adwaita/GTK4 theme with Phosphor vector icons (#6)

### Miscellaneous Tasks

- Release v0.1.8

### Other

- V0.2.0
## [0.1.6] - 2026-06-23

### Miscellaneous Tasks

- Release v0.1.6

### Other

- V0.1.6 (#7)
## [0.1.5] - 2026-06-23

### Features

- *(notes)* SQLite-based editable notes with CRUD, insert/delete/reimport, emoji font fix (#4) (#5)

### Miscellaneous Tasks

- Release v0.1.5
## [0.1.4] - 2026-06-23

### Features

- *(notes)* Add presenter notes support

### Miscellaneous Tasks

- Release v0.1.4

### Other

- V0.2.0

### Refactor

- *(render)* Make renderer synchronous, remove PDF support
## [0.1.3] - 2026-06-22

### Miscellaneous Tasks

- Release v0.1.3

### Other

- V0.2.0
## [0.1.2] - 2026-06-22

### Miscellaneous Tasks

- Release v0.1.2
## [0.1.1] - 2026-06-22

### Miscellaneous Tasks

- Release v0.1.1
