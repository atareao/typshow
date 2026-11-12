//! UI de Typshow basada en GPUI.
//!
//! Sustituye progresivamente la UI `egui`/`eframe`. Reutiliza el dominio
//! (`TypstWorld`, `AppState`, `Notes`, `NotesDb`) y reescribe solo la capa de UI.
//! Durante la migración convive con el binario egui (`typshow`).

pub mod audience;
pub mod deck;
pub mod edit;
pub mod keys;
pub mod presenter;
pub mod theme;
