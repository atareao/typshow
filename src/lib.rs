mod app;
mod fullscreen;
mod presenter;
mod render;
mod theme;

mod notes;
mod notes_db;
mod notes_egui;

pub mod gpui_ui;

pub use app::AppState;
pub use app::SharedState;
pub use notes::Notes;
pub use notes_db::NotesDb;
pub use render::Renderer;
pub use render::TypstWorld;

// Re-export for main.rs convenience
pub use presenter::TypshowApp;
