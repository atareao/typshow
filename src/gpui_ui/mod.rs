//! UI de Typshow basada en GPUI.
//!
//! Es la única capa de UI del crate. Reutiliza el dominio (`TypstWorld`,
//! `Notes`, `NotesDb`) y el binario `typshow` arranca el presentador desde aquí.

pub mod audience;
pub mod config;
pub mod deck;
pub mod edit;
pub mod icons;
pub mod keys;
pub mod presenter;
pub mod theme;

/// `true` si cerrar la ventana `closing` debe terminar la app, es decir, si es
/// la ventana del presentador (la de la audiencia NO debe terminar la app).
///
/// Es un predicado puro para poder testearlo sin abrir ventanas; el cableado
/// real vive en `main.rs`, dentro de `App::on_window_closed`.
pub fn quits_app_on_close(closing: gpui::WindowId, presenter: gpui::WindowId) -> bool {
    closing == presenter
}

#[cfg(test)]
mod tests {
    use super::quits_app_on_close;
    use gpui::WindowId;

    /// Ventana del presentador (identificador arbitrario pero estable).
    fn presenter_id() -> WindowId {
        WindowId::from(1u64)
    }

    /// Cualquier otra ventana (p. ej. la de audiencia).
    fn audience_id() -> WindowId {
        WindowId::from(2u64)
    }

    #[test]
    fn closing_the_presenter_quits_the_app() {
        assert!(
            quits_app_on_close(presenter_id(), presenter_id()),
            "cerrar la ventana del presentador debe terminar la app"
        );
    }

    #[test]
    fn closing_the_audience_does_not_quit_the_app() {
        assert!(
            !quits_app_on_close(audience_id(), presenter_id()),
            "cerrar la audiencia no debe terminar la app (solo desactiva la proyección)"
        );
    }
}
