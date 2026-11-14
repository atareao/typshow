# Tasks

## 1. Config (RED)

- [ ] 1.1 Tests del módulo `config`: round-trip `save_theme_to`/`load_theme_from` con fichero temporal; `load_theme_from` de fichero inexistente → `None`; contenido inválido → `None`; `save_theme_to` crea el directorio padre.

## 2. Config (GREEN)

- [ ] 2.1 `src/gpui_ui/config.rs` con `config_dir`, `theme_path`, `load_theme_from`, `save_theme_to`; registrar el módulo en `src/gpui_ui/mod.rs`.

## 3. Deck (RED → GREEN)

- [ ] 3.1 Test: `Deck::load_with(ruta_typ, Some(ruta_tema))` carga el tema guardado; `toggle_theme` lo guarda; un segundo `load_with` lo recupera.
- [ ] 3.2 Añadir `theme_path: Option<PathBuf>` a `Deck`; cargar en `load`; persistir en `toggle_theme`.

## 4. Cierre (REFACTOR)

- [ ] 4.1 `cargo fmt --all -- --check` y `rustup run 1.100.0 cargo clippy --all-targets -- -D warnings` limpios.
- [ ] 4.2 `cargo test` en verde.
- [ ] 4.3 Archivar el change (`openspec archive`) al mergear.
