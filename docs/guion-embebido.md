# Guion embebido en el `.typ`

Issue #52 · Plan `plans/PLAN-010.md` · Change OpenSpec `openspec/changes/guion-embebido-en-typ/`.

## Qué es

El guion del presentador viaja **dentro del propio `.typ`** como comentarios de
línea de Typst, de forma que es invisible en el PDF (y en la ventana de
audiencia) pero typshow lo lee y lo muestra como nota de la página.

## Formato del bloque

```
== Hubo un día

// >>> typshow:notes
// [0:00 – 1:00]
//
// Hubo un día. Y no lo digo como figura literaria…
// >>> typshow:notes:end
```

- Apertura `// >>> typshow:notes`, cierre `// >>> typshow:notes:end` (exactos).
- Cada línea del contenido es un comentario `//`; al decodificar se elimina el
  `//` y, como máximo, un espacio siguiente. Una línea `//` sola equivale a una
  línea vacía del Markdown (se conserva).
- El bloque se **ancla** al heading de slide (`=` o `==`) inmediatamente
  anterior. Un bloque sin heading previo, sin cierre o anidado se **descarta**
  con `tracing::warn!`; la carga nunca aborta.

## Cómo funciona

| Pieza | Dónde | Qué hace |
|---|---|---|
| `parse_embedded_blocks` | `src/notes_db.rs` | Escáner de líneas que extrae y ancla los bloques (`EmbeddedBlock { heading, markdown, start_line, end_line }`). |
| `embedded_block_ranges` | `src/notes_db.rs` | Rangos de líneas de los bloques; `extract_typ_segments` los excluye para que el guion no se cuele como contenido de nota (fix del *leak*). |
| `build_page_seeds` | `src/notes_db.rs` | Calcula página → (heading, contenido). Con bloques, el contenido es el Markdown del bloque anclado; sin bloques, se conserva el comportamiento previo (contenido de la diapositiva). |
| `migrate_from_typ` / `sync_from_blocks` | `src/notes_db.rs` | Sembrado inicial si la BD está vacía; en aperturas posteriores, **reimportación selectiva por hash**. |
| `content_hash` | `src/notes_db.rs` | Hash estable (`DefaultHasher` a hex) del Markdown de un bloque. |
| `Notes::reimport` | `src/notes.rs` | No vacía la tabla: delega en la reimportación selectiva (no existe ya `clear_all`). |
| `render_exported_typ` / `export_to_typ` | `src/notes_db.rs` | Vuelca las notas de la BD a los bloques del `.typ` preservando el resto del fichero (inserta, actualiza o elimina el bloque de cada página). |
| `script_section_bodies` / `embed_script_into_typ` / `guion_embed_files` | `src/notes_db.rs` | Conversión de un guion Markdown (`## …`) a bloques, idempotente. |
| `Deck::export_notes` + botón «Exportar» | `src/gpui_ui/deck.rs`, `src/gpui_ui/presenter.rs` | Acción del presentador que vuelca las notas actuales al `.typ`. |
| Subcomando CLI | `src/main.rs` | `typshow guion-embed <guion.md> <presentacion.typ>`. |

## Esquema (`notes`)

Añadidas a la tabla del sidecar `<nombre>.typshow.db`:

- `source_hash TEXT` — hash del bloque del que se sembró la nota (NULL si no
  procede de un bloque).
- `edited INTEGER NOT NULL DEFAULT 0` — 1 si el usuario editó la nota en el
  panel.
- `heading_edited INTEGER NOT NULL DEFAULT 0` — 1 si el usuario editó el
  *título* a mano; lo protege de la reimportación.

`init_schema` migra bases existentes con `pragma_table_info` + `ALTER TABLE ADD
COLUMN` (idempotente, sin pérdida de filas).

### Reglas de reimportación selectiva

Contenido:

| Estado de la página | Bloque | Acción |
|---|---|---|
| No existe fila | hay | Sembrar (`source_hash = h`, `edited = 0`) |
| `source_hash == h` | igual | Nada |
| `source_hash != h`, `edited = 0` | cambió | Refrescar contenido y `source_hash` |
| `source_hash != h`, `edited = 1` | cambió | Conservar la nota + `warn!` de divergencia |
| `source_hash = Some(_)`, `edited = 0` | desapareció | Vaciar la nota y `source_hash = NULL` |
| `source_hash = Some(_)`, `edited = 1` | desapareció | Conservar la nota + `warn!` de divergencia |
| `source_hash = None` | no hay | Nada |

Heading: se refresca desde el documento salvo que el usuario lo haya editado a
mano (`heading_edited = 1`).

Nunca se borran filas y, si nada cambió, no se escribe ninguna fila (ni siquiera
`updated_at`).

## Mapeo página ↔ bloque

Igual que el histórico: página 0 es la portada (`title:` de `slides.with(...)`),
y cada heading `=`/`==` ocupa exactamente una página, en orden. Por tanto la
n-ésima sección `## …` del guion va a la página n.

Una sección `## …` con el cuerpo vacío **no** inserta bloque; si esa página ya
tenía uno, el bloque se elimina (el guion es la fuente de verdad) y la sección no
cuenta en el total de bloques escritos.

## Cómo usarlo

```bash
# Convertir el guion Markdown en bloques embebidos (idempotente)
typshow guion-embed guion-episodio.md episodio.typ

# Abrir la presentación: el presentador muestra el guion como notas
typshow episodio.typ
```

Desde el presentador: «Reimportar» refresca solo las páginas no editadas cuyo
bloque cambió, y «Exportar» vuelca las notas al `.typ`.

## Decisiones

- El bloque se lee con un escáner de líneas, no con un parser de Typst
  completo: los centinelas son comentarios.
- **Anclaje posicional**: cada bloque guarda la *línea* de su heading ancla, así
  que dos títulos con el mismo texto (p. ej. dos `== Repaso`) no desvían el
  guion a la página equivocada.
- **Bloque anidado**: se descarta el exterior y vale el interior (no se anidan).
- **Centinelas en el contenido**: una línea de la nota que sea exactamente
  `>>> typshow:notes` (o `:end`) se escapa con `\` al escribirla y se desescapa
  al leerla, para que el round-trip sea exacto. Por el mismo motivo se escapan
  las líneas que ya empiezan por `\`: en un bloque, una línea escrita a mano que
  empiece por `\` y no sea inequívoca se conserva tal cual.
- Hash **FNV-1a de 64 bits** implementado en el crate (no `DefaultHasher`): el
  hash se persiste en `source_hash` y `DefaultHasher` no garantiza el mismo
  resultado entre versiones de Rust.
- Finales de línea: el export detecta el final dominante del fichero y lo
  respeta (un `.typ` con CRLF sigue con CRLF).
- El `.typ` no se reescribe si el export no cambia nada (`export_to_typ`
  compara antes de escribir).

## Limitaciones conocidas

- **Primera apertura con bloques sobre una BD anterior**: las filas creadas por
  versiones previas a esta feature no tienen columna `edited` (nace con valor 0),
  así que en la primera apertura con bloques esas notas **sí** se refrescan y
  pierden la edición local. A partir de esa apertura ya quedan protegidas.
- **Título**: el heading solo está protegido si el usuario lo guarda con el
  editor de título del panel (`heading_edited = 1`). Si el heading venía del
  `.typ` y el usuario no lo tocó, se reescribe con el documento en cada
  reimportación (comportamiento deseado).
- El `source_hash` de una nota editada **no** se actualiza aunque el bloque
  cambie: la divergencia se vuelve a avisar en cada apertura hasta que el usuario
  exporte (o vacíe) la nota.
- Los ficheros con finales de línea mezclados se normalizan al final dominante.

## Follow-ups

- Verificación manual sobre un episodio real (guion invisible en PDF, visible en
  el presentador) — cubierto a nivel de píxel por el test
  `guion_embebido_no_aparece_en_el_pdf` de `tests/pipeline_test.rs`.
- Archivar el change OpenSpec (`openspec archive`) al mergear el issue #52.
