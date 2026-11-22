# Botón «Importar guion» en el panel de notas

Issue #67 · Plan `plans/PLAN-014.md` · Change OpenSpec `openspec/changes/boton-importar-guion/`.

## Qué es

Permite importar el guion al `.typ` **desde la propia app**, sin pasar por la terminal. Un
botón en el panel de notas abre un diálogo para el `.md`, ejecuta el mismo embebido que el
subcomando `guion-embed` (modo por rango #59 o por secciones `##` #52), escribe los bloques
en el `.typ` abierto y refresca las notas del presentador.

## Ficheros implicados

- `src/gpui_ui/deck.rs` — `Deck::import_guion(path) -> Result<usize, String>`.
- `src/gpui_ui/presenter.rs` — botón `notes-importar-guion`, `import_guion_dialog`,
  `notes_status` (aviso efímero, `NOTES_STATUS_TTL = 3 s`).
- `src/gpui_ui/icons.rs` — icono `FILE_ARROW_DOWN` (Phosphor `U+E232`).
- `src/lib.rs` — reexporta `embed_script_into_typ` como `crate::embed_script_into_typ`.
- `tests/` (unit en `deck.rs`) — `import_guion_*`.

## Cómo funciona

1. Botón visible cuando hay un `.typ` cargado y no se está editando. Abre `rfd::AsyncFileDialog`
   filtrado a `.md` (patrón de «Abrir», con guardia de reentrada `dialog_open`).
2. `Deck::import_guion`:
   - sin documento ⇒ `Err("No hay documento cargado")`;
   - lee el guion y calcula `(new_typ, written) = embed_script_into_typ(script, typ_source)`;
   - si `written == 0` ⇒ `Err` con aviso y **no** escribe;
   - escribe `new_typ` en el fichero (solo si cambió), actualiza `typ_content`;
   - `reimport_notes()` con la **reimportación selectiva por hash** (refresca las no editadas,
     **preserva** las editadas a mano; guarda antes la edición en curso).
3. Aviso **efímero** en el panel (3 s): «N bloques importados» o el mensaje de error/aviso.

## Efecto

- El flujo queda dentro de la app: elegir guion → notas pobladas por página.
- Idempotente: reimportar el mismo guion no duplica bloques ni reescribe el fichero.
- No cambia el render (los bloques son comentarios) ni el CLI `guion-embed`.

## Variables / configuración

- `NOTES_STATUS_TTL` (duración del aviso efímero; 3 s). Sin cambios de esquema.

## Cómo habilitar / desplegar

`cargo install --path .` y abrir una presentación. El botón aparece en el panel de notas.

## Decisiones

- **En el panel de notas**, junto a Guardar/Nueva/Borrar, con **icono + etiqueta** como el resto.
- **Embeber en el `.typ` abierto + refrescar notas** (no un modo «solo memoria»): el guion
  queda en el fichero, como con el CLI.
- **Preservar** las notas editadas a mano (reimportación selectiva ya existente).
- **Aviso efímero** en vez de un mensaje persistente.

## Limitaciones / follow-ups

- Requiere un `.typ` ya abierto; no ofrece elegir el destino si no hay ninguno.
- El diálogo usa `rfd` (ya dependencia); en Wayland/Niri depende del portal en uso.

## Limitaciones conocidas (observaciones menores de auditoría, no bloqueantes)

- **Estado parcial si falla la reimportación tras escribir:** si `reimport_notes()` falla
  **después** de escribir el `.typ` y actualizar `typ_content`, el disco/la fuente quedan
  nuevos pero las notas sin refrescar (no hay rollback). Fallo poco probable.
- **Feedback de reimportación:** un segundo import sin cambios reporta «N bloques
  importados» aunque no se reescriba nada (`written` cuenta bloques generados). El fichero
  sí queda intacto (idempotente).
- La rama «sin documento cargado» del diálogo es defensiva: el botón solo se dibuja con un
  `.typ` cargado.
