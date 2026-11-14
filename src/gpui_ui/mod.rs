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
