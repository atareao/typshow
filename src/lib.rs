mod app;
mod render;
mod theme;
mod presenter;
mod fullscreen;

mod notes;
mod notes_db;
pub use app::AppState;
pub use app::SharedState;
pub use notes::Notes;
pub use notes_db::NotesDb;
pub use render::Renderer;
pub use render::TypstWorld;

// Re-export for main.rs convenience
pub use presenter::TypshowApp;