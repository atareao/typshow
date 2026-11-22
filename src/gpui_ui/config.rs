//! Persistencia de la preferencia de tema (claro/oscuro) en el directorio de
//! configuración XDG.

use std::path::{Path, PathBuf};

use super::theme::{NOTES_PANEL_MAX_WIDTH, NOTES_PANEL_MIN_WIDTH};

/// Nombre del subdirectorio de la app dentro de la configuración del usuario.
const APP_DIR: &str = "typshow";

/// Nombre del fichero que guarda la preferencia de tema.
const THEME_FILE: &str = "theme";

/// Nombre del fichero que guarda el ancho del panel de notas.
pub const NOTES_WIDTH_FILE: &str = "notes_width";

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

/// `true` si `width` es finito y cae dentro de `[NOTES_PANEL_MIN_WIDTH,
/// NOTES_PANEL_MAX_WIDTH]` (límites inclusivos).
fn notes_width_in_range(width: f32) -> bool {
    width.is_finite() && (NOTES_PANEL_MIN_WIDTH..=NOTES_PANEL_MAX_WIDTH).contains(&width)
}

/// Recorta `width` a los límites del panel. El recorte se aplica **solo al
/// guardar**; al cargar, un valor fuera de rango se descarta (ver
/// [`load_notes_width_from`]).
fn sanitize_notes_width(width: f32) -> f32 {
    if width.is_finite() {
        width.clamp(NOTES_PANEL_MIN_WIDTH, NOTES_PANEL_MAX_WIDTH)
    } else {
        NOTES_PANEL_MIN_WIDTH
    }
}

/// Ruta del fichero de ancho del panel de notas (`<config_dir>/notes_width`).
/// `None` si no hay config_dir.
pub fn notes_width_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join(NOTES_WIDTH_FILE))
}

/// Lee el ancho del panel de notas de `path`. `None` si el fichero no existe, su
/// contenido no es un número finito o el valor cae **fuera** del rango válido;
/// en esos casos el llamante usa `NOTES_PANEL_WIDTH` como ancho por defecto. El
/// recorte a los límites es cosa del guardado ([`save_notes_width_to`]).
pub fn load_notes_width_from(path: &Path) -> Option<f32> {
    let raw: f32 = std::fs::read_to_string(path).ok()?.trim().parse().ok()?;
    notes_width_in_range(raw).then_some(raw)
}

/// Escribe `width` (recortado a los límites) en `path`, creando el directorio
/// padre si hace falta. Contenido: el ancho seguido de un salto de línea.
pub fn save_notes_width_to(path: &Path, width: f32) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{}\n", sanitize_notes_width(width)))
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

    #[test]
    fn round_trip_ancho_notas() {
        let dir = temp_dir("notes_roundtrip");
        let path = dir.join("notes_width");

        save_notes_width_to(&path, 480.).unwrap();
        assert_eq!(load_notes_width_from(&path), Some(480.));

        save_notes_width_to(&path, 333.5).unwrap();
        assert_eq!(load_notes_width_from(&path), Some(333.5));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ancho_notas_ausente_none() {
        let dir = temp_dir("notes_missing");
        let path = dir.join("notes_width");

        assert_eq!(load_notes_width_from(&path), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ancho_notas_invalido_none() {
        let dir = temp_dir("notes_invalid");
        let path = dir.join("notes_width");

        for content in ["ancho\n", "", "NaN\n", "inf\n", "480px\n"] {
            std::fs::write(&path, content).unwrap();
            assert_eq!(
                load_notes_width_from(&path),
                None,
                "contenido {content:?} debe leerse como ausente"
            );
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ancho_notas_fuera_de_rango_none() {
        let dir = temp_dir("notes_out_of_range");
        let path = dir.join("notes_width");

        // Fuera de rango (alto o bajo) se descarta al cargar: el llamante cae al
        // ancho por defecto. El recorte es cosa del guardado.
        std::fs::write(&path, "10000\n").unwrap();
        assert_eq!(load_notes_width_from(&path), None);

        std::fs::write(&path, "10\n").unwrap();
        assert_eq!(load_notes_width_from(&path), None);

        // Los límites exactos sí son válidos.
        std::fs::write(&path, "200\n").unwrap();
        assert_eq!(load_notes_width_from(&path), Some(NOTES_PANEL_MIN_WIDTH));

        std::fs::write(&path, "640\n").unwrap();
        assert_eq!(load_notes_width_from(&path), Some(NOTES_PANEL_MAX_WIDTH));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ancho_notas_recorta_y_crea_dir() {
        let dir = temp_dir("notes_save");
        let path = dir.join("sub").join("notes_width");

        save_notes_width_to(&path, 10_000.).unwrap();
        assert!(path.exists(), "el fichero de ancho debe existir");
        // El valor se recorta ANTES de escribir: en disco queda `640`, no `10000`.
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "640\n");
        assert_eq!(load_notes_width_from(&path), Some(NOTES_PANEL_MAX_WIDTH));

        save_notes_width_to(&path, 0.).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "200\n");
        assert_eq!(load_notes_width_from(&path), Some(NOTES_PANEL_MIN_WIDTH));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
