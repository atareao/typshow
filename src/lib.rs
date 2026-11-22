mod notes;
mod notes_db;
mod render;

pub mod gpui_ui;

pub use notes::Notes;
pub use notes_db::NotesDb;
/// Embebido de guion reutilizado por la UI (`Deck::import_guion`).
pub(crate) use notes_db::embed_script_into_typ;
pub use notes_db::{EmbeddedBlock, export_to_typ, guion_embed_files, parse_embedded_blocks};
pub use render::TypstWorld;
