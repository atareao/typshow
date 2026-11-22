# Proposal

## Why

El guion embebido (#52) mapea la **n-ésima sección `## …`** de un guion Markdown a la
**página n** del `.typ` (1:1 por orden). Eso funciona en 8 de ~34 episodios. El resto
usa guiones de **grano grueso**: no tienen secciones `##`, sino **bloques por tramos de
tiempo** delimitados por separadores `─` y cabeceras tipo
`2:00 – 6:30  📋 BLOQUE 1 — ¿Por qué systemd timers…?`. Para esos guiones
`typshow guion-embed` escribe **cero bloques** (no encuentra ningún `##`), así que el
guion no viaja al `.typ`. Issue #59.

## What Changes

- **Parser de bloques por tramos**: nueva función que trocea un guion de grano grueso en
  sus bloques (separadores `─` + cabecera con marca de tiempo), en orden, ignorando el
  preámbulo anterior al primer bloque y el material posterior al último.
- **Rangos de página por sección `=`**: cada sección `=` del `.typ` cubre un rango de
  páginas (su página y las siguientes hasta la anterior a la siguiente `=`); las
  diapositivas `==` de dentro caen en ese rango.
- **Mapeo ordinal/proporcional** bloques ↔ secciones `=` (función pura): n-ésimo bloque ↔
  n-ésima sección; si el número difiere, reparto proporcional monótono que cubre todas
  las secciones.
- **Escritura en rango**: el Markdown de un bloque se escribe como bloque embebido en
  **cada página** de su rango (replicado), de modo que el presentador lo ve mientras
  presenta ese tramo.
- **Modo automático con fallback**: si el guion trae secciones `##`, se mantiene el
  comportamiento actual (1:1); si no, se activa el mapeo por rango. El pipeline del
  podcast no necesita cambios de invocación.

## Capabilities

### New Capabilities

- `notas`: nuevo requisito «Mapeo por rango de guiones de grano grueso».

### Modified Capabilities

- `notas`: el requisito «Conversión del guion Markdown a bloques embebidos» se amplía con
  el modo por rango como *fallback* cuando no hay secciones `##`.

## Impact

- `src/notes_db.rs`: parser de bloques por tramos; rangos por sección `=`; mapeo puro;
  `embed_script_ranges_into_typ`; fallback en `guion_embed_files`/`embed_script_into_typ`.
- `src/main.rs`: el subcomando `guion-embed` no cambia de firma (modo automático).
- Sin cambios de esquema SQLite. Sin cambios de UI (las notas se muestran por página).
- Pipeline del podcast: sin cambios de invocación (fuera de este repo; verificación como
  follow-up).
