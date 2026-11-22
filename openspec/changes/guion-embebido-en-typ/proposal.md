# Proposal

## Why

Hoy typshow siembra las notas del presentador desde el sidecar SQLite
(`<nombre>.typshow.db`): al abrirlo vacío, `notes_db.rs::migrate_from_typ` trocea
el `.typ` por headings (`= / == / ===`) y `#pagebreak` y guarda **el contenido de
la diapositiva** como nota de esa página. Es decir, la nota es un eco de lo que ya
se ve en pantalla.

El guion real de cada episodio vive en un fichero Markdown aparte (`guion-…​.md`)
que el `.typ` desconoce. Nada garantiza que viajen juntos, y el presentador no ve
nunca el guion. El issue #52 recoge la petición de que el guion sea **importable
directamente en la presentación**.

La clave arquitectónica es que Typst **no renderiza los comentarios**: un guion
vivido dentro de comentarios del `.typ` es invisible en el PDF (y por tanto en la
ventana de audiencia) pero puede ser leído por typshow. El `.typ` pasa a ser el
contenedor único de diapositiva **y** guion, versionado y portable.

## What Changes

- Se define un **bloque de guion embebido** en el `.typ`, delimitado por
  centinelas en comentarios de línea (`// >>> typshow:notes` … `// >>> typshow:notes:end`).
- Se **ancla** cada bloque al heading de slide inmediatamente anterior (`=`/`==`),
  de modo que el mapeo bloque → página sea posicional y sin ambigüedad.
- El **sembrado** de notas pasa a usar el Markdown del bloque anclado cuando existe
  (con *fallback* al comportamiento actual si el documento no trae bloques). Se
  corrige además el *leak* actual: las líneas `//` del bloque MUST quedar excluidas
  del contenido de segmento, para que no se cuelen como texto de la nota.
- **Reimport selectivo por hash**: el esquema de `notes` gana `source_hash` y un
  indicador de edición. Al abrir se refrescan solo las páginas cuyo bloque cambió
  y que no tengan ediciones locales; nunca se hace `clear_all`. Las páginas editadas
  en el panel se preservan y las divergencias se registran.
- **Export bidireccional**: una acción vuelca las notas actuales de la BD a los
  bloques del `.typ`, preservando byte a byte el resto del fichero.
- **Conversor** `guion.md → bloques .typ` (subcomando de CLI) para cerrar el ciclo
  de autoría desde el Markdown fuente.

## Capabilities

### New Capabilities

- `notas`: define el formato del guion embebido, su anclaje a páginas, el sembrado,
  la reimportación selectiva por hash, el export al `.typ` y el conversor desde
  Markdown. Es una capacidad de dominio (persistencia + formato), separada de la UI.

### Modified Capabilities

<!-- Ninguna: el panel de notas de `gpui-ui` no cambia su comportamiento visible
     salvo el origen del contenido sembrado, que se especifica en `notas`. -->

## Non-goals

- No se cambia el renderizado de páginas ni el pipeline de Typst.
- No se introduce una UI nueva de gestión de bloques: el panel de notas sigue
  siendo el punto de edición.
- No se implementa un parser de Typst completo: el bloque se lee con un escáner de
  líneas (los centinelas son comentarios, no código).
- No se soportan bloques en cualquier posición arbitraria; MUST ir precedidos de un
  heading de slide.
- No se sincroniza en tiempo real mientras el fichero está abierto por otro editor
  (la relectura ocurre al abrir/recargar).

## Impact

- `src/notes_db.rs`: parser del bloque embebido y su anclaje; ampliación del esquema
  (`source_hash`, `edited`); sembrado desde bloques; reimport selectivo (sustituir el
  `clear_all` de `reimport`); export de la BD al `.typ`.
- `src/notes.rs`: orquestación del sembrado/reimport/export y del *fallback*.
- `src/main.rs`: subcomando de conversión `guion.md → bloques .typ`.
- Tests: unitarios del parser/anclaje/hash y de la conversión; integración del ciclo
  sembrado → edición → reimport → export sobre un `.typ` temporal.
- Sin dependencias nuevas previsibles (hash con `std::hash` o un hash estable ligero).
