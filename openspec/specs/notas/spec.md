# notas Specification

## Purpose
TBD - created by archiving change guion-embebido-en-typ. Update Purpose after archive.

## Requirements

### Requirement: Formato del guion embebido

El `.typ` SHALL poder contener bloques de guion embebido delimitados por los
centinelas `// >>> typshow:notes` (apertura) y `// >>> typshow:notes:end` (cierre),
compuestos exclusivamente por comentarios de línea de Typst. El contenido del bloque
SHALL interpretarse como Markdown, donde cada línea de comentario equivale a una
línea Markdown (se elimina el prefijo `//` y, como máximo, un espacio siguiente).
El bloque MUST NOT alterar el renderizado del documento.

#### Scenario: Lectura de un bloque embebido

- **GIVEN** un `.typ` con un bloque `// >>> typshow:notes` … `// >>> typshow:notes:end`
- **WHEN** typshow lee el documento
- **THEN** obtiene el Markdown del bloque, conservando líneas vacías y formato inline

#### Scenario: El bloque no se renderiza

- **GIVEN** un `.typ` con un bloque de guion embebido
- **WHEN** Typst compila el documento a PDF
- **THEN** ninguna línea del guion aparece en el PDF ni en la ventana de audiencia

#### Scenario: Bloque sin cerrar

- **GIVEN** un `.typ` con un centinela de apertura sin su cierre
- **WHEN** typshow lee el documento
- **THEN** descarta ese bloque y registra un aviso, sin abortar la carga

### Requirement: Anclaje del bloque a una página

Cada bloque de guion SHALL asociarse a la diapositiva cuyo heading (`=` o `==`)
precede inmediatamente al centinela de apertura. Un bloque sin heading anterior MUST
ignorarse y registrarse como aviso.

#### Scenario: Bloque anclado a su slide

- **GIVEN** un `== Título` seguido de un bloque de guion embebido
- **WHEN** typshow construye el mapeo bloque → página
- **THEN** el Markdown del bloque se asocia a la página de ese `== Título`

#### Scenario: Varios slides, cada uno con su bloque

- **GIVEN** un `.typ` con tres `==` y un bloque tras cada uno
- **WHEN** typshow lee el documento
- **THEN** cada página recibe el Markdown de su propio bloque, sin cruces

#### Scenario: Bloque sin heading

- **GIVEN** un bloque de guion al principio del fichero, antes de cualquier heading
- **WHEN** typshow lee el documento
- **THEN** ese bloque se descarta y se registra un aviso

### Requirement: Sembrado inicial desde el guion embebido

Al abrir una presentación cuya base de datos de notas esté vacía, typshow SHALL sembrar
cada página con el Markdown del bloque anclado a su slide. Si el documento contiene al
menos un bloque embebido, el sembrado MUST usar los bloques; si no contiene ninguno,
MUST conservarse el comportamiento previo (contenido de la diapositiva). Las líneas de
los bloques embebidos MUST quedar excluidas del contenido de segmento, para que no se
cuelen como texto de nota.

#### Scenario: Sembrado desde bloques

- **GIVEN** un `.typ` con bloques embebidos y una BD de notas vacía
- **WHEN** se abre la presentación
- **THEN** cada página muestra como nota el Markdown de su bloque, no el contenido del slide

#### Scenario: Documento sin bloques

- **GIVEN** un `.typ` sin bloques embebidos y una BD vacía
- **WHEN** se abre la presentación
- **THEN** el sembrado se comporta como hasta ahora (contenido de la diapositiva)

#### Scenario: Sin fuga de líneas de comentario

- **GIVEN** un `.typ` con un bloque embebido
- **WHEN** typshow extrae los segmentos/notas
- **THEN** ninguna línea `//` del bloque aparece como contenido de nota o de segmento

### Requirement: Reimportación selectiva por hash

La tabla de notas SHALL almacenar, por página, un `source_hash` del Markdown embebido
del que se sembró y un indicador de edición local. Al abrir o recargar, typshow SHALL
comparar el hash embebido con el almacenado y SHALL refrescar únicamente las páginas
cuyo bloque cambió y que no tengan ediciones locales. typshow MUST NOT vaciar la base
de datos ni sobrescribir notas editadas por el usuario; las divergencias se registrarán.

#### Scenario: Bloque modificado en una página no editada

- **GIVEN** una nota sembrada sin ediciones locales y un bloque embebido que cambió
- **WHEN** se recarga la presentación
- **THEN** esa página se refresca con el nuevo Markdown y se actualiza su `source_hash`

#### Scenario: Bloque modificado en una página editada

- **GIVEN** una nota editada por el usuario y un bloque embebido que cambió
- **WHEN** se recarga la presentación
- **THEN** se conserva la nota editada y se registra la divergencia, sin sobrescribirla

#### Scenario: Sin cambios

- **GIVEN** notas cuyo `source_hash` coincide con el bloque embebido actual
- **WHEN** se recarga la presentación
- **THEN** no se modifican filas ni contenido

#### Scenario: Recarga no destructiva

- **GIVEN** una base de datos con notas y ediciones
- **WHEN** se dispara la reimportación
- **THEN** ninguna fila se borra por completo (no se usa `clear_all`)

### Requirement: Exportación de las notas al `.typ`

typshow SHALL ofrecer una acción que vuelque las notas actuales a los bloques
embebidos del `.typ`, preservando el resto del fichero sin cambios. Para cada página
con nota no vacía SHALL escribir o actualizar su bloque tras el heading correspondiente;
para páginas sin nota SHALL eliminar el bloque si existiera.

#### Scenario: Exportar una nota nueva

- **GIVEN** una página con nota en la BD y sin bloque embebido en el `.typ`
- **WHEN** el usuario exporta
- **THEN** el `.typ` gana un bloque embebido con esa nota tras el heading de la página

#### Scenario: Actualizar un bloque existente

- **GIVEN** una página con nota editada y un bloque embebido desactualizado
- **WHEN** el usuario exporta
- **THEN** el bloque se reescribe con la nota actual y el resto del `.typ` queda intacto

#### Scenario: Nota vacía

- **GIVEN** una página sin nota y con un bloque embebido en el `.typ`
- **WHEN** el usuario exporta
- **THEN** el bloque se elimina y el resto del fichero permanece igual

### Requirement: Conversión del guion Markdown a bloques embebidos

typshow SHALL ofrecer un subcomando que convierta un guion Markdown en bloques embebidos
dentro de un `.typ`. Con secciones `## …`, cada sección SHALL asociarse a un heading de
slide por orden (1:1). La detección de secciones `## …` SHALL ignorar las que aparezcan
dentro de un **bloque de código cercado** (``` … ```) y las que tengan **indentación de
código** (4 o más espacios), de modo que el Markdown embebido en un guion no se confunda
con cabeceras. Sin secciones `##` pero con bloques por tramos de tiempo, SHALL aplicarse
el **mapeo por rango** a las secciones `=` (requisito «Mapeo por rango de guiones de grano
grueso»). La conversión MUST ser idempotente: reejecutarla sobre el mismo `.typ` MUST NOT
duplicar bloques.

#### Scenario: Conversión por orden de secciones

- **GIVEN** un `guion.md` con N secciones `## …` y un `.typ` con N headings de slide
- **WHEN** se ejecuta el subcomando
- **THEN** cada sección se escribe como bloque embebido tras su heading, en orden

#### Scenario: Conversión idempotente

- **GIVEN** un `.typ` que ya contiene los bloques generados
- **WHEN** se vuelve a ejecutar el subcomando
- **THEN** los bloques existentes se actualizan y no se duplican

#### Scenario: Secciones dentro de bloques de código se ignoran

- **GIVEN** un guion con cabeceras `## …` reales y, además, líneas `## …` dentro de un bloque de código cercado
- **WHEN** se detectan las secciones del guion
- **THEN** las líneas del bloque de código NO cuentan como secciones

#### Scenario: Cabecera con indentación de código no es sección

- **GIVEN** un guion con una línea `    ## …` (4 espacios) fuera de un fence
- **WHEN** se detectan las secciones del guion
- **THEN** esa línea NO cuenta como sección

#### Scenario: Un guion con bloques de código mantiene el modo por rango

- **GIVEN** una escaleta con bloques por tramos de tiempo y Markdown embebido en un bloque de código
- **WHEN** se ejecuta el subcomando
- **THEN** se aplica el mapeo por rango (no la vía `##`) y se escribe el guion completo

### Requirement: Mapeo por rango de guiones de grano grueso

Cuando un guion no contiene secciones `## …` pero sí **bloques por tramos de tiempo**
(delimitados por separadores de línea `─` y una cabecera con marca de tiempo y título,
p. ej. `2:00 – 6:30  📋 BLOQUE 1 — …`), typshow SHALL trocearlo en esos bloques en orden
e ignorar el preámbulo anterior al primer bloque y el material posterior al último. Los
bloques SHALL mapearse a las **secciones `=` del `.typ`**, donde cada sección cubre un
**rango de páginas** (su página y las siguientes hasta la anterior a la siguiente `=`).
El Markdown del bloque SHALL escribirse como bloque embebido en **cada página** de su
rango. El mapeo SHALL ser **ordinal** (n-ésimo bloque ↔ n-ésima sección `=`); cuando el
número de bloques y de secciones difiera, SHALL repartirse **proporcionalmente** de forma
monótona y cubriendo todas las secciones. La conversión MUST ser idempotente y el bloque
MUST NOT alterar el renderizado del `.typ`.

#### Scenario: Detección de bloques por tramos de tiempo

- **GIVEN** un guion con cabeceras `0:00 – 2:00  🎬 INTRO`, `2:00 – 6:30  📋 BLOQUE 1 — …`
  separadas por líneas `─`
- **WHEN** typshow lee el guion
- **THEN** obtiene los bloques en orden, cada uno con su cuerpo, e ignora el preámbulo
  anterior al primer bloque

#### Scenario: Mapeo ordinal a secciones `=`

- **GIVEN** un guion con B bloques y un `.typ` con S secciones `=` donde B = S
- **WHEN** se ejecuta la conversión por rango
- **THEN** el bloque n se escribe como bloque embebido en todas las páginas de la sección `=` n

#### Scenario: Reparto proporcional

- **GIVEN** un guion con B bloques y un `.typ` con S secciones `=` donde B ≠ S
- **WHEN** se ejecuta la conversión por rango
- **THEN** cada bloque se asigna a una sección por reparto proporcional, de forma monótona
  y cubriendo todas las secciones

#### Scenario: Replicación en cada página del rango

- **GIVEN** una sección `=` que cubre las páginas p…q
- **WHEN** se escribe el bloque de esa sección
- **THEN** el Markdown del bloque aparece embebido en cada una de las páginas p…q

#### Scenario: Fallback a secciones `##`

- **GIVEN** un guion con secciones `## …`
- **WHEN** se ejecuta la conversión
- **THEN** se aplica el comportamiento previo (mapeo 1:1 por orden) y no el mapeo por rango

#### Scenario: Guion sin bloques reconocibles

- **GIVEN** un guion sin secciones `##` ni cabeceras de tramos válidas
- **WHEN** se ejecuta la conversión
- **THEN** no se escribe ningún bloque y se registra un aviso, sin abortar

#### Scenario: El guion de grano grueso no se renderiza

- **GIVEN** un `.typ` con bloques generados por mapeo por rango
- **WHEN** Typst compila el documento a PDF
- **THEN** ninguna línea del guion aparece en el PDF ni en la ventana de audiencia

### Requirement: Importación del guion desde la aplicación

La aplicación SHALL poder importar un guion Markdown al `.typ` cargado ejecutando el mismo
embebido que el subcomando `guion-embed` (modo por rango o por secciones `##`): leer el
guion, escribir los bloques embebidos en el fichero del `.typ`, actualizar la fuente en
memoria y **reimportar** las notas. La reimportación SHALL ser la **selectiva por hash**:
refresca las notas no editadas y **preserva** las editadas a mano. Si el guion no tiene
secciones `##` ni bloques por tramos, MUST NOT escribirse nada y SHALL informarse del aviso.
La importación MUST ser idempotente: reimportar el mismo guion sobre el mismo `.typ` no
duplica bloques.

#### Scenario: Importar un guion de grano grueso

- **GIVEN** un `.typ` cargado y un guion con bloques por tramos de tiempo
- **WHEN** se importa el guion
- **THEN** el `.typ` se actualiza con los bloques y las notas del presentador se reimportan

#### Scenario: Se preservan las notas editadas a mano

- **GIVEN** un `.typ` con notas, algunas editadas a mano
- **WHEN** se importa un guion
- **THEN** las notas editadas se preservan y las no editadas se refrescan

#### Scenario: Guion sin secciones ni tramos

- **GIVEN** un guion sin secciones `##` ni cabeceras de tramos válidas
- **WHEN** se importa
- **THEN** no se escribe ningún bloque y se informa del aviso, sin abortar

#### Scenario: Importación idempotente

- **GIVEN** un `.typ` que ya contiene los bloques generados por el guion
- **WHEN** se vuelve a importar el mismo guion
- **THEN** los bloques se actualizan y no se duplican
