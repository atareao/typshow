# gpui-ui Specification

## Purpose
Interfaz de Typshow basada en GPUI que sustituye a egui: ventana de presentador y
ventana de audiencia sincronizadas, render de páginas Typst, navegación, panel de notas,
tema e integración con el sistema.

## Requirements

### Requirement: Ventana de presentador
La app SHALL mostrar una ventana de presentador con la página actual, la previsualización
de la página siguiente y una barra de navegación.

#### Scenario: Paneles Actual y Siguiente
- **WHEN** la app arranca con un documento válido
- **THEN** el presentador muestra la página actual y, debajo, la página siguiente

#### Scenario: Sin página siguiente
- **WHEN** la página actual es la última
- **THEN** el panel de "Siguiente" indica el fin de la presentación en lugar de una página

#### Scenario: Barra de navegación
- **WHEN** el presentador se muestra
- **THEN** ofrece los controles Inicio, Anterior, Siguiente y Final, deshabilitados cuando
  no tienen efecto (p. ej. "Anterior" en la primera página)

### Requirement: Ventana de audiencia a pantalla completa
La app SHALL abrir una ventana de audiencia a pantalla completa que muestre la página
actual.

#### Scenario: Audiencia a pantalla completa
- **WHEN** la proyección está activa
- **THEN** existe una ventana a pantalla completa con la página actual, sin decoraciones

#### Scenario: Cierre de la audiencia
- **WHEN** el usuario pulsa Esc en la ventana de audiencia
- **THEN** la ventana de audiencia se cierra y la proyección se desactiva

### Requirement: Estado sincronizado entre ventanas
Las dos ventanas SHALL compartir el estado de la presentación (página actual, tema) y
reflejar los cambios de forma coherente.

#### Scenario: Cambio de página sincronizado
- **WHEN** el presentador cambia de página
- **THEN** la ventana de audiencia muestra la nueva página

#### Scenario: Cambio de tema sincronizado
- **WHEN** se alterna el tema
- **THEN** ambas ventanas cambian de tema de forma coherente

### Requirement: Render de páginas Typst

La app SHALL compilar el documento Typst y renderizar cada página como imagen,
preservando los colores: cada píxel SHALL rasterizarse con los canales correctos
(convertidos al formato BGRA que espera GPUI), sin intercambiar rojo y azul.

#### Scenario: Página renderizada

- **WHEN** el documento Typst compila correctamente
- **THEN** la página actual se muestra como imagen en las ventanas

#### Scenario: Colores preservados en todo píxel

- **WHEN** una página contiene una forma de color plano (p. ej. amarillo)
- **THEN** todos sus píxeles se rasterizan con el color correcto (el amarillo no se
  convierte en cian)

#### Scenario: Error de compilación

- **WHEN** el documento Typst no compila
- **THEN** la app muestra un mensaje de error en lugar de una página en blanco

### Requirement: Navegación por teclado y ratón
La app SHALL permitir navegar entre páginas con el teclado y el ratón.

#### Scenario: Navegación con teclado
- **WHEN** el usuario pulsa las teclas de avance/retroceso (flechas, `h`/`j`/`k`/`l`,
  `g`/`G`, espacio, Home/End)
- **THEN** la página actual cambia respetando los límites del documento

#### Scenario: Navegación con clic
- **WHEN** el usuario hace clic izquierdo o derecho sobre la página
- **THEN** la página retrocede o avanza respectivamente

### Requirement: Abrir documento
La app SHALL permitir abrir un fichero `.typ` desde la interfaz.

#### Scenario: Abrir fichero
- **WHEN** el usuario elige "Abrir" y selecciona un `.typ`
- **THEN** el documento se carga y se muestra su primera página

### Requirement: Toggle de proyección

La app SHALL permitir mostrar u ocultar la ventana de audiencia. Además, `Esc` SHALL cerrar
la proyección siempre que esté activa, con independencia de que haya una edición de notas en
curso.

#### Scenario: Alternar proyección

- **WHEN** el usuario activa/desactiva la proyección
- **THEN** la ventana de audiencia se abre o se cierra en consecuencia

#### Scenario: Esc cierra la proyección sin edición

- **GIVEN** la proyección está activa y no hay ninguna edición en curso
- **WHEN** el usuario pulsa `Esc`
- **THEN** la ventana de audiencia se cierra y la proyección queda inactiva

#### Scenario: Esc cierra la proyección durante la edición

- **GIVEN** la proyección está activa y el usuario está editando una nota
- **WHEN** el usuario pulsa `Esc`
- **THEN** la ventana de audiencia se cierra y la proyección queda inactiva
- **AND** la edición permanece abierta (no se descarta el texto que el usuario escribía)

#### Scenario: Esc sin proyección cancela la edición

- **GIVEN** la proyección no está activa y el usuario está editando una nota
- **WHEN** el usuario pulsa `Esc`
- **THEN** la edición se cancela y no ocurre nada más

#### Scenario: Esc sin proyección ni edición

- **GIVEN** la proyección no está activa y el usuario no está editando
- **WHEN** el usuario pulsa `Esc`
- **THEN** no ocurre nada

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

### Requirement: Edición de texto con selección e IME
Los editores de texto de la ventana de presentador (cuerpo de la nota y título) SHALL
mantener cursor, selección con dirección y rango marcado de composición IME, SHALL
intercambiar rangos con GPUI en UTF-16 (`EntityInputHandler`) y SHALL permitir copiar y
cortar la selección al portapapeles.

#### Scenario: Selección con teclado
- **GIVEN** un editor con foco y el cursor situado en una posición
- **WHEN** el usuario mantiene Shift y pulsa las flechas izquierda/derecha
- **THEN** la selección se extiende carácter a carácter desde el ancla, el cursor queda en
  el extremo opuesto y el resaltado refleja la dirección de la selección

#### Scenario: Selección con ratón
- **GIVEN** un editor con texto
- **WHEN** el usuario hace clic o arrastra el ratón, con o sin Shift
- **THEN** sin arrastre el cursor salta al punto pulsado (con Shift extiende la selección) y
  con arrastre se selecciona el rango entre el ancla y la posición actual

#### Scenario: Borrar o insertar sobre una selección
- **GIVEN** un editor con un rango seleccionado no vacío
- **WHEN** el usuario escribe, pega, pulsa Backspace o pulsa Delete
- **THEN** el texto seleccionado se sustituye por lo insertado (o se elimina si no se inserta
  nada) y el cursor queda al final del texto insertado

#### Scenario: Seleccionar todo
- **WHEN** el usuario pulsa el atajo de "seleccionar todo" (Ctrl+A / Cmd+A)
- **THEN** la selección cubre todo el texto y el cursor queda al final

#### Scenario: Copiar y cortar al portapapeles
- **GIVEN** un editor con foco y un rango seleccionado no vacío
- **WHEN** el usuario pulsa el atajo de copiar (Ctrl+C / Cmd+C) o el de cortar (Ctrl+X / Cmd+X)
- **THEN** la selección se escribe en el portapapeles y, solo al cortar, se borra del texto; en el
  cuerpo de la nota ese cambio la marca como sucia, mientras que en el título el texto se persiste
  al pulsar Enter

#### Scenario: Copiar sin selección
- **GIVEN** un editor con foco y la selección colapsada
- **WHEN** el usuario pulsa el atajo de copiar
- **THEN** el evento se consume sin escribir el portapapeles ni modificar el documento

#### Scenario: Composición IME en curso
- **GIVEN** el usuario está componiendo texto con un método de entrada (IME)
- **WHEN** el IME entrega texto marcado (`replace_and_mark_text_in_range`)
- **THEN** el texto provisional se inserta y se muestra subrayado como rango marcado, y la
  selección provisional del IME se respeta acotada a la longitud del texto insertado

#### Scenario: Confirmación de la composición IME
- **GIVEN** un editor con un rango marcado activo
- **WHEN** el IME confirma o cancela la composición
- **THEN** al confirmar desaparece el rango marcado y el texto queda como definitivo; al
  cancelar el texto provisional se sustituye por lo que indique el IME (`unmark_text`)

#### Scenario: Mapeo UTF-16 con acentos y emoji
- **GIVEN** un texto con caracteres fuera del BMP o acentuados (p. ej. `café`, `A🙂Z`)
- **WHEN** GPUI consulta la longitud, un rango o la posición del cursor en unidades UTF-16 y
  luego los devuelve
- **THEN** la conversión byte↔UTF-16 es exacta y los viajes de ida y vuelta preservan los
  offsets originales en frontera de carácter

### Requirement: Tema claro/oscuro estilo Adwaita

La app SHALL ofrecer un tema claro y uno oscuro con estética GTK4/Adwaita, y SHALL
recordar la última preferencia de tema elegida por el usuario entre sesiones.

#### Scenario: Alternar tema

- **WHEN** el usuario alterna el tema
- **THEN** los colores de la interfaz cambian entre claro y oscuro de forma coherente

#### Scenario: Persistencia del tema

- **WHEN** el usuario elige un tema (claro u oscuro) y vuelve a abrir la app
- **THEN** la app arranca con ese mismo tema

#### Scenario: Sin preferencia guardada

- **WHEN** no hay ninguna preferencia de tema guardada (o el fichero no es válido)
- **THEN** la app arranca en el tema claro por defecto

### Requirement: Iconos
La app SHALL mostrar iconos en los controles de la interfaz.

#### Scenario: Iconos visibles
- **WHEN** se muestran los controles (abrir, tema, proyección, notas, navegación)
- **THEN** cada control muestra su icono correspondiente

### Requirement: Binario por defecto
Al completarse la migración, GPUI SHALL ser el framework de UI por defecto de Typshow.

#### Scenario: Binario por defecto en GPUI
- **WHEN** la migración finaliza
- **THEN** el binario `typshow` usa GPUI y `egui`/`eframe` se han retirado de las
  dependencias
