use egui::{Color32, RichText, Ui};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use crate::notes_db::NotesDb;
use tracing::{info, warn};

pub enum MdElement {
    Heading(u32, String),
    Paragraph(String),
    ListItem(String),
    CodeBlock(String),
    HorizontalRule,
    EmptyLine,
}

pub struct Notes {
    pub db: Option<NotesDb>,
    pub current_page: usize,
    pub note_content: Option<String>,
    pub heading: Option<String>,
    pub heading_editing: bool,
    pub heading_edit_buffer: String,
    pub elements: Vec<MdElement>,
    pub editing: bool,
    pub edit_buffer: String,
    pub dirty: bool,
    pub scroll_ratio: f32,
    pub total_pages: usize,
}

impl Notes {
    pub fn new() -> Self {
        Self {
            db: None,
            current_page: 0,
            note_content: None,
            heading: None,
            heading_editing: false,
            heading_edit_buffer: String::new(),
            elements: Vec::new(),
            editing: false,
            edit_buffer: String::new(),
            dirty: false,
            scroll_ratio: 0.0,
            total_pages: 0,
        }
    }

    pub fn with_db(
        typ_path: &str,
        typ_content: &str,
        total_pages: usize,
    ) -> Self {
        let mut notes = Self::new();
        notes.total_pages = total_pages;

        match NotesDb::open(typ_path) {
            Ok(db) => {
                if let Err(e) = db.migrate_from_typ(typ_content, total_pages) {
                    warn!("Error en migración de notas: {}", e);
                }
                notes.db = Some(db);
                notes.load_page(0);
            }
            Err(e) => {
                warn!("No se pudo abrir base de datos de notas: {}", e);
            }
        }

        notes
    }

    pub fn load_page(&mut self, page: usize) {
        self.current_page = page;
        self.editing = false;
        self.heading_editing = false;
        self.heading_edit_buffer.clear();

        if let Some(ref db) = self.db {
            match db.load(page) {
                Ok(record) => {
                    self.note_content = Some(record.content.clone());
                    self.heading = record.heading;
                    self.elements = if record.content.is_empty() {
                        Vec::new()
                    } else {
                        Self::parse_md(&record.content)
                    };
                }
                Err(e) => {
                    warn!("Error cargando nota de página {}: {}", page, e);
                    self.note_content = None;
                    self.heading = None;
                    self.elements = Vec::new();
                }
            }
        } else {
            self.note_content = None;
            self.heading = None;
            self.elements = Vec::new();
        }
    }

    pub fn save_current(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref db) = self.db {
            db.save(self.current_page, &self.edit_buffer)?;
            self.note_content = Some(self.edit_buffer.clone());
            self.elements = if self.edit_buffer.is_empty() {
                Vec::new()
            } else {
                Self::parse_md(&self.edit_buffer)
            };
            self.dirty = false;
            self.editing = false;
            info!("Nota guardada para página {}", self.current_page + 1);
        }
        Ok(())
    }

    pub fn start_edit(&mut self) {
        self.edit_buffer = self.note_content.clone().unwrap_or_default();
        self.editing = true;
        self.dirty = false;
    }

    pub fn cancel_edit(&mut self) {
        self.edit_buffer.clear();
        self.editing = false;
        self.dirty = false;
    }

    pub fn delete_current(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref db) = self.db {
            db.delete(self.current_page)?;
            self.note_content = None;
            self.heading = None;
            self.elements = Vec::new();
            self.edit_buffer.clear();
            self.editing = false;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn insert_note(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref db) = self.db {
            db.shift_up(self.current_page)?;
            if let Some(ref heading) = self.heading {
                db.set_heading(self.current_page, Some(heading))?;
            }
            if let Some(ref content) = self.note_content {
                db.save(self.current_page, content)?;
            } else {
                db.insert_note(self.current_page)?;
            }
            self.total_pages += 1;
            self.load_page(self.current_page);
            self.start_edit();
        }
        Ok(())
    }

    pub fn reimport(&mut self, typ_content: &str, total_pages: usize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref db) = self.db {
            db.clear_all()?;
            db.migrate_from_typ(typ_content, total_pages)?;
            self.total_pages = total_pages;
            self.load_page(self.current_page);
        }
        Ok(())
    }

    pub fn set_heading(&mut self, heading: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref db) = self.db {
            db.set_heading(self.current_page, heading)?;
            self.heading = heading.map(|s| s.to_string());
        }
        Ok(())
    }

    pub fn start_edit_heading(&mut self) {
        self.heading_edit_buffer = self.heading.clone().unwrap_or_default();
        self.heading_editing = true;
    }

    pub fn save_heading(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let new_heading = if self.heading_edit_buffer.trim().is_empty() {
            None
        } else {
            Some(self.heading_edit_buffer.trim().to_string())
        };
        self.set_heading(new_heading.as_deref())?;
        self.heading = new_heading;
        self.heading_editing = false;
        self.heading_edit_buffer.clear();
        Ok(())
    }

    pub fn cancel_edit_heading(&mut self) {
        self.heading_editing = false;
        self.heading_edit_buffer.clear();
    }

    pub fn update_scroll_target(&mut self, current_page: usize, total_pages: usize) {
        self.scroll_ratio = if total_pages > 1 {
            current_page as f32 / (total_pages - 1) as f32
        } else {
            0.0
        };
    }

    pub fn has_content(&self) -> bool {
        self.db.is_some()
    }

    pub fn draw(&self, ui: &mut Ui) {
        if self.elements.is_empty() {
            ui.colored_label(Color32::GRAY, "Las notas están vacías");
            return;
        }

        for element in &self.elements {
            match element {
                MdElement::Heading(level, text) => {
                    let size = match level {
                        1 => 22.0, 2 => 18.0, 3 => 16.0,
                        4 => 14.0, 5 => 13.0, _ => 12.0,
                    };
                    ui.add_space(8.0);
                    ui.label(RichText::new(text).size(size).strong());
                    ui.add_space(4.0);
                }
                MdElement::Paragraph(text) => {
                    ui.label(Self::format_inline(text));
                    ui.add_space(4.0);
                }
                MdElement::ListItem(text) => {
                    ui.label(format!("  •  {}", Self::format_inline(text).text()));
                    ui.add_space(2.0);
                }
                MdElement::CodeBlock(code) => {
                    egui::Frame::NONE
                        .fill(ui.style().visuals.extreme_bg_color)
                        .inner_margin(egui::Margin::symmetric(6, 4))
                        .show(ui, |ui| {
                            ui.monospace(code);
                        });
                    ui.add_space(4.0);
                }
                MdElement::HorizontalRule => {
                    ui.separator();
                    ui.add_space(4.0);
                }
                MdElement::EmptyLine => {
                    ui.add_space(8.0);
                }
            }
        }
    }

    fn parse_md(source: &str) -> Vec<MdElement> {
        let mut elements = Vec::new();
        let parser = Parser::new(source);
        let mut pending_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let lvl = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    elements.push(MdElement::Heading(lvl, String::new()));
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(MdElement::Heading(_, text)) = elements.last_mut() {
                        if !pending_text.is_empty() {
                            text.push_str(&pending_text);
                            pending_text.clear();
                        }
                    }
                }
                Event::Start(Tag::Paragraph) => {
                    pending_text.clear();
                }
                Event::End(TagEnd::Paragraph) => {
                    if !pending_text.trim().is_empty() {
                        elements.push(MdElement::Paragraph(pending_text.trim().to_string()));
                    }
                    pending_text.clear();
                }
                Event::Start(Tag::Item) => {
                    pending_text.clear();
                }
                Event::End(TagEnd::Item) => {
                    if !pending_text.trim().is_empty() {
                        elements.push(MdElement::ListItem(pending_text.trim().to_string()));
                    }
                    pending_text.clear();
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    pending_text.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    elements.push(MdElement::CodeBlock(pending_text.trim().to_string()));
                    pending_text.clear();
                }
                Event::Start(Tag::Emphasis) => {
                    pending_text.push_str("*");
                }
                Event::End(TagEnd::Emphasis) => {
                    pending_text.push_str("*");
                }
                Event::Start(Tag::Strong) => {
                    pending_text.push_str("**");
                }
                Event::End(TagEnd::Strong) => {
                    pending_text.push_str("**");
                }
                Event::Code(t) => {
                    pending_text.push_str(&format!("`{}`", t));
                }
                Event::Text(t) => {
                    pending_text.push_str(&t);
                }
                Event::SoftBreak | Event::HardBreak => {
                    pending_text.push(' ');
                }
                Event::Rule => {
                    elements.push(MdElement::HorizontalRule);
                }
                _ => {}
            }
        }

        if !pending_text.trim().is_empty() {
            elements.push(MdElement::Paragraph(pending_text.trim().to_string()));
        }

        elements
    }

    fn format_inline(text: &str) -> RichText {
        let has_bold = text.contains("**");
        let has_code = text.contains('`');
        let clean = text.replace("**", "").replace("`", "");
        let mut rt = RichText::new(clean);
        if has_bold {
            rt = rt.strong();
        }
        if has_code {
            rt = rt.code();
        }
        rt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let n = Notes::new();
        assert!(n.db.is_none());
        assert!(n.note_content.is_none());
        assert!(!n.editing);
    }

    #[test]
    fn test_parse_md_empty_string_after_whitespace() {
        let e1 = Notes::parse_md("   \n\n  ");
        let e2 = Notes::parse_md("");
        assert!(e1.is_empty());
        assert!(e2.is_empty());
    }

    #[test]
    fn test_parse_md_paragraphs() {
        let e = Notes::parse_md("Hello world.\n\nSecond para.");
        assert_eq!(e.len(), 2);
        assert!(matches!(e[0], MdElement::Paragraph(ref t) if t == "Hello world."));
        assert!(matches!(e[1], MdElement::Paragraph(ref t) if t == "Second para."));
    }

    #[test]
    fn test_parse_md_headings_and_paragraphs() {
        let e = Notes::parse_md("# Title\n\nSome text.");
        assert_eq!(e.len(), 2);
        assert!(matches!(e[0], MdElement::Heading(1, ref t) if t == "Title"));
        assert!(matches!(e[1], MdElement::Paragraph(ref t) if t == "Some text."));
    }

    #[test]
    fn test_parse_md_bold_and_code() {
        let e = Notes::parse_md("**bold** and `code` here");
        assert_eq!(e.len(), 1);
        if let MdElement::Paragraph(ref t) = e[0] {
            assert_eq!(t, "**bold** and `code` here");
        } else {
            panic!("Expected paragraph");
        }
    }

    #[test]
    fn test_parse_md_unordered_list() {
        let e = Notes::parse_md("- Item A\n- Item B\n- Item C");
        assert_eq!(e.len(), 3);
        for item in &e {
            assert!(matches!(item, MdElement::ListItem(_)));
        }
    }

    #[test]
    fn test_parse_md_code_block() {
        let e = Notes::parse_md("```\nlet x = 1;\n```");
        assert_eq!(e.len(), 1);
        if let MdElement::CodeBlock(ref c) = e[0] {
            assert_eq!(c, "let x = 1;");
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn test_parse_md_horizontal_rule() {
        let e = Notes::parse_md("Before\n\n---\n\nAfter");
        assert!(e.iter().any(|el| matches!(el, MdElement::HorizontalRule)));
    }

    #[test]
    fn test_format_inline_plain() {
        let rt = Notes::format_inline("hello world");
        assert_eq!(rt.text(), "hello world");
    }

    #[test]
    fn test_format_inline_bold() {
        let rt = Notes::format_inline("**bold text**");
        assert!(!rt.text().contains("**"));
        assert!(rt.text().contains("bold text"));
    }

    #[test]
    fn test_format_inline_code() {
        let rt = Notes::format_inline("use `fn()` here");
        assert!(rt.text().contains("fn()"));
        assert!(!rt.text().contains("`"));
    }

    #[test]
    fn test_format_inline_bold_and_code() {
        let rt = Notes::format_inline("**`x`**");
        assert_eq!(rt.text(), "x");
    }

    #[test]
    fn test_start_edit_copies_content() {
        let mut n = Notes::new();
        n.note_content = Some("Hello".to_string());
        n.start_edit();
        assert!(n.editing);
        assert_eq!(n.edit_buffer, "Hello");
    }

    #[test]
    fn test_cancel_edit_clears() {
        let mut n = Notes::new();
        n.note_content = Some("Hello".to_string());
        n.start_edit();
        n.cancel_edit();
        assert!(!n.editing);
        assert!(n.edit_buffer.is_empty());
    }

    #[test]
    fn test_update_scroll_target_first_page() {
        let mut n = Notes::new();
        n.update_scroll_target(0, 10);
        assert_eq!(n.scroll_ratio, 0.0);
    }

    #[test]
    fn test_update_scroll_target_middle() {
        let mut n = Notes::new();
        n.update_scroll_target(5, 10);
        assert!((n.scroll_ratio - 0.555).abs() < 0.01);
    }

    #[test]
    fn test_update_scroll_target_last_page() {
        let mut n = Notes::new();
        n.update_scroll_target(9, 10);
        assert!((n.scroll_ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_update_scroll_target_single_page() {
        let mut n = Notes::new();
        n.update_scroll_target(0, 1);
        assert_eq!(n.scroll_ratio, 0.0);
    }

    #[test]
    fn test_load_page_no_db() {
        let mut n = Notes::new();
        n.load_page(5);
        assert_eq!(n.current_page, 5);
        assert!(n.note_content.is_none());
        assert!(!n.editing);
    }
}