//! Persistencia de la preferencia de tema (claro/oscuro) en el directorio de
//! configuración XDG.

use std::path::{Path, PathBuf};

/// Nombre del subdirectorio de la app dentro de la configuración del usuario.
const APP_DIR: &str = "typshow";

/// Nombre del fichero que guarda la preferencia de tema.
const THEME_FILE: &str = "theme";

/// Directorio de configuración de la app: `$XDG_CONFIG_HOME/typshow` o, si no,
/// `$HOME/.config/typshow`. `None` si no hay variable de entorno utilizable.
pub fn config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join(APP_DIR))
}

/// Ruta del fichero de tema (`<config_dir>/theme`). `None` si no hay config_dir.
pub fn theme_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join(THEME_FILE))
}

/// Lee la preferencia de `path`. `Some(true)`=oscuro, `Some(false)`=claro,
/// `None` si el fichero no existe o su contenido no es `dark`/`light`.
pub fn load_theme_from(path: &Path) -> Option<bool> {
    match std::fs::read_to_string(path).ok()?.trim() {
        "dark" => Some(true),
        "light" => Some(false),
        _ => None,
    }
}

/// Escribe la preferencia en `path`, creando el directorio padre si hace falta.
/// Contenido: `dark\n` o `light\n`.
pub fn save_theme_to(path: &Path, dark: bool) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, if dark { "dark\n" } else { "light\n" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Directorio temporal único por test (evita colisiones entre tests paralelos).
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("typshow_config_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn round_trip_dark_and_light() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("theme");

        save_theme_to(&path, true).unwrap();
        assert_eq!(load_theme_from(&path), Some(true));

        save_theme_to(&path, false).unwrap();
        assert_eq!(load_theme_from(&path), Some(false));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_is_none() {
        let dir = temp_dir("missing");
        let path = dir.join("theme");

        assert_eq!(load_theme_from(&path), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_content_is_none() {
        let dir = temp_dir("invalid");
        let path = dir.join("theme");
        std::fs::write(&path, "azul\n").unwrap();

        assert_eq!(load_theme_from(&path), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_creates_parent_directory() {
        let dir = temp_dir("parent");
        let path = dir.join("sub").join("theme");

        save_theme_to(&path, true).unwrap();

        assert!(path.exists(), "el fichero de tema debe existir");
        assert_eq!(load_theme_from(&path), Some(true));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
