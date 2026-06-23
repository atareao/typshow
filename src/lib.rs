mod app;
mod render;
mod theme;
mod presenter;
mod fullscreen;

mod notes;
pub use app::AppState;
pub use app::SharedState;
pub use notes::Notes;
pub use render::Renderer;
pub use render::TypstWorld;

// Re-export for main.rs convenience
pub use presenter::TypshowApp;