# PLAN-014 — Botón «Importar guion» en el panel de notas

- **Issue:** #67 · **Rama:** `feature/boton-importar-guion` (base `development`) · **OpenSpec:** `openspec/changes/boton-importar-guion/`

## Objetivo

Importar el guion al `.typ` desde la propia app: botón en el panel de notas que abre un
diálogo para el `.md`, ejecuta el embebido (rango o `##`), escribe los bloques y refresca
las notas, preservando las editadas a mano.

## Diseño

- `Deck::import_guion(path) -> Result<usize, String>`: leer guion → `embed_script_into_typ`
  → escribir `.typ` → actualizar `typ_content` → `reimport_notes` (selectivo por hash).
- `presenter.rs`: botón `notes-importar-guion` en `notes_panel` + `import_guion_dialog`
  (patrón de `open_file_dialog`, filtro `.md`, guardia de reentrada).
- Feedback en el panel (nº de bloques / aviso / error).

## Tareas (TDD, espejo de tasks.md)

- [x] 1. `Deck::import_guion` + tests (embebe+reimporta, preserva editadas, sin bloques, idempotencia, sin documento).
- [x] 2. Botón + diálogo + feedback en `presenter.rs`.
- [x] 3. Auditoría APROBADA, fmt/clippy/test limpios, doc y archivado. Verificación manual UI confirmada por el usuario.

## Verificación

```bash
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
```

## Estado

- [x] Spec aprobado (icono+etiqueta y aviso efímero).
