# Tasks

## 1. Test que reproduce el fallo (RED)

- [ ] 1.1 Test de `rgba_to_bgra` con un buffer de varios píxeles; verificar que FALLA con el código actual
- [ ] 1.2 Test de render de una página de color plano (amarillo) comprobando que TODOS los píxeles salen con el color correcto; verificar que FALLA

## 2. Corrección (GREEN)

- [ ] 2.1 `rgba_to_bgra`: iterar `data.chunks_exact_mut(4)` aplicando `swap_rgba_pa_to_bgra` a cada píxel; `cargo test` en verde

## 3. Cierre (REFACTOR)

- [ ] 3.1 `cargo fmt --all -- --check` y `cargo clippy --all-targets -- -D warnings` limpios
- [ ] 3.2 Marcar tareas y archivar el change (`openspec archive`) al mergear
