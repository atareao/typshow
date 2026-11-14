# Persistir la preferencia de tema

## Qué es

La app recuerda si el usuario eligió tema **claro** u **oscuro** y arranca con él la próxima vez.
Sin preferencia guardada (o con un fichero inválido), arranca en claro por defecto.

## Ficheros implicados

- `src/gpui_ui/config.rs` — resolución de rutas y lectura/escritura de la preferencia.
- `src/gpui_ui/deck.rs` — `Deck` carga y guarda el tema.
- `src/gpui_ui/mod.rs` — registra el módulo `config`.

## Cómo funciona

- Ubicación del fichero: `$XDG_CONFIG_HOME/typshow/theme`, con `$HOME/.config/typshow/theme`
  como respaldo. Contenido: `dark\n` o `light\n`.
- **Carga**: `Deck::load` resuelve la ruta (`config::theme_path`) y lee la preferencia
  (`config::load_theme_from`); si no hay fichero o no es `dark`/`light`, deja el tema claro.
- **Guardado**: `Deck::toggle_theme` (llamado por el botón "Tema" y por la tecla `t`)
  invierte el flag y escribe la preferencia (best-effort: si falla, se registra un `warn!`
  y la UI no se rompe).
- `Deck::load_with` es la variante con la ruta inyectada, para tests sin tocar el entorno.

## Variables / configuración

- `XDG_CONFIG_HOME` (opcional): si es una ruta absoluta, se usa como base. Si no, se usa `$HOME/.config`.

## Cómo habilitar / desplegar

Es comportamiento automático; entra con el binario. No requiere migración.

## Decisiones

- Sin dependencias nuevas (`dirs`/`serde`/`toml`): formato mínimo de una línea.
- Escritura best-effort: un fallo de disco no impide cambiar de tema en la sesión.
- Ruta inyectable en `Deck` para no acoplar los tests al entorno.

## Follow-ups

- Ninguno.
