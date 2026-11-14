## MODIFIED Requirements

### Requirement: Panel de notas

La app SHALL mostrar y permitir editar las notas del presentador asociadas a la página
actual, con persistencia en la base de datos. Al mostrarlas SHALL aplicar el formato Markdown
inline completo: negrita (`**…**`), cursiva (`*…*` / `_…_`) y código (`` `…` ``), sin mostrar
los marcadores.

#### Scenario: Ver nota de la página

- **WHEN** la página actual tiene una nota
- **THEN** el panel de notas muestra su contenido (con formato Markdown)

#### Scenario: Formato inline

- **WHEN** una nota contiene `**negrita**`, `*cursiva*` o `` `código` ``
- **THEN** el texto se pinta en negrita, en cursiva o en monoespaciado respectivamente, sin
  los marcadores

#### Scenario: Marcadores sin cerrar

- **WHEN** una nota contiene un marcador inline sin cerrar (p. ej. `*suelto` o `**abierto`)
- **THEN** se conserva el texto literal (no se pierde contenido)

#### Scenario: Editar y guardar nota

- **WHEN** el usuario edita una nota y guarda
- **THEN** el cambio se persiste y se refleja al volver a esa página

#### Scenario: Reimportar notas desde el .typ

- **WHEN** el usuario pulsa "Reimportar"
- **THEN** las notas se reconstruyen a partir del documento Typst
