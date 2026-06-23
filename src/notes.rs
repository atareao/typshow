use egui::{Color32, RichText, Ui};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use tracing::info;

#[derive(Clone, Debug)]
pub struct HeadingInfo {
    pub level: u32,
    pub text: String,
    pub line: usize,
}

pub enum MdElement {
    Heading(u32, String),
    Paragraph(String),
    ListItem(String),
    CodeBlock(String),
    HorizontalRule,
    EmptyLine,
}

pub struct Notes {
    pub path: Option<String>,
    pub content: Option<String>,
    pub typ_headings: Vec<HeadingInfo>,
    pub md_headings: Vec<HeadingInfo>,
    pub elements: Vec<MdElement>,
    pub line_count: usize,
    pub typ_line_count: usize,
    pub scroll_ratio: f32,
}

impl Notes {
    pub fn new() -> Self {
        Self {
            path: None,
            content: None,
            typ_headings: Vec::new(),
            md_headings: Vec::new(),
            elements: Vec::new(),
            line_count: 0,
            typ_line_count: 0,
            scroll_ratio: 0.0,
        }
    }

    pub fn load(typ_path: &str, typ_content: &str) -> Self {
        let md_path = Self::derive_md_path(typ_path);
        info!("Notes - Buscando notas en: {:?}", md_path);

        let content = std::fs::read_to_string(&md_path).ok();
        let md_content = content.as_deref().unwrap_or("");

        let md_headings = Self::parse_md_headings(md_content);
        let typ_headings = Self::parse_typ_headings(typ_content);
        let elements = if md_content.is_empty() { Vec::new() } else { Self::parse_md(md_content) };
        let line_count = md_content.lines().count();
        let typ_line_count = typ_content.lines().count();

        info!(
            "Notes - {} headings en .typ, {} headings en .md, {} líneas",
            typ_headings.len(),
            md_headings.len(),
            line_count
        );

        Self {
            path: Some(md_path),
            content,
            typ_headings,
            md_headings,
            elements,
            line_count,
            typ_line_count,
            scroll_ratio: 0.0,
        }
    }

    fn derive_md_path(typ_path: &str) -> String {
        let p = std::path::Path::new(typ_path);
        p.with_extension("md").to_string_lossy().to_string()
    }

    fn parse_typ_headings(source: &str) -> Vec<HeadingInfo> {
        let mut headings = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            let level = if trimmed.starts_with("==== ") { 4 }
            else if trimmed.starts_with("=== ") { 3 }
            else if trimmed.starts_with("== ") { 2 }
            else if trimmed.starts_with("= ") { 1 }
            else { continue };

            let text = trimmed[level..].trim().to_string();
            headings.push(HeadingInfo { level: level as u32, text, line: i });
        }
        headings
    }

    fn parse_md_headings(source: &str) -> Vec<HeadingInfo> {
        let mut headings = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            let mut level = 0u32;
            for ch in trimmed.chars() {
                if ch == '#' { level += 1; } else { break; }
            }
            if level == 0 || level > 6 { continue }
            if !trimmed.as_bytes().get(level as usize).is_some_and(|&b| b == b' ') { continue }

            let text = trimmed[level as usize + 1..].trim().to_string();
            headings.push(HeadingInfo { level, text, line: i });
        }
        headings
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

    pub fn update_scroll_target(&mut self, current_page: usize, total_pages: usize) {
        if total_pages == 0 || self.line_count == 0 {
            self.scroll_ratio = 0.0;
            return;
        }

        if current_page == 0 {
            self.scroll_ratio = 0.0;
            return;
        }

        if !self.typ_headings.is_empty() && !self.md_headings.is_empty() {
            let md_count = self.md_headings.len();
            let typ_count = self.typ_headings.len();
            let heading_count = md_count.min(typ_count);

            if heading_count > 0 {
                let slot_count = heading_count - 1;
                let md_idx = if total_pages > 1 && slot_count > 0 {
                    let raw = ((current_page - 1) * slot_count) / (total_pages - 1);
                    raw.min(slot_count)
                } else {
                    0
                };
                self.scroll_ratio = md_idx as f32 / slot_count.max(1) as f32;
                return;
            }
        }

        self.scroll_ratio = current_page as f32 / total_pages.max(1) as f32;
    }

    pub fn draw(&self, ui: &mut Ui) {
        if self.content.is_none() {
            let msg = match &self.path {
                Some(p) => format!("No se encontraron notas en:\n{}", p),
                None => "No hay notas disponibles".to_string(),
            };
            ui.colored_label(Color32::GRAY, msg);
            return;
        }

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
    fn test_derive_md_path_basic() {
        assert_eq!(Notes::derive_md_path("/tmp/pres.typ"), "/tmp/pres.md");
    }

    #[test]
    fn test_derive_md_path_no_extension() {
        assert_eq!(Notes::derive_md_path("/tmp/pres"), "/tmp/pres.md");
    }

    #[test]
    fn test_derive_md_path_nested() {
        assert_eq!(Notes::derive_md_path("slides/deck.typ"), "slides/deck.md");
    }

    #[test]
    fn test_parse_typ_headings_levels() {
        let src = "= H1\n== H2\n=== H3\n==== H4\nother\n== H2bis";
        let h = Notes::parse_typ_headings(src);
        assert_eq!(h.len(), 5);
        assert_eq!(h[0].level, 1); assert_eq!(h[0].text, "H1");
        assert_eq!(h[1].level, 2); assert_eq!(h[1].text, "H2");
        assert_eq!(h[2].level, 3); assert_eq!(h[2].text, "H3");
        assert_eq!(h[3].level, 4); assert_eq!(h[3].text, "H4");
        assert_eq!(h[4].level, 2); assert_eq!(h[4].text, "H2bis");
    }

    #[test]
    fn test_parse_typ_headings_empty() {
        assert!(Notes::parse_typ_headings("plain text\nno headers").is_empty());
    }

    #[test]
    fn test_parse_typ_headings_trailing_space() {
        let h = Notes::parse_typ_headings("= Heading with trailing  ");
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].text, "Heading with trailing");
    }

    #[test]
    fn test_parse_md_headings_levels() {
        let src = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\nplain";
        let h = Notes::parse_md_headings(src);
        assert_eq!(h.len(), 6);
        assert_eq!(h[0].level, 1); assert_eq!(h[0].text, "H1");
        assert_eq!(h[1].level, 2); assert_eq!(h[1].text, "H2");
        assert_eq!(h[2].level, 3); assert_eq!(h[2].text, "H3");
        assert_eq!(h[3].level, 4); assert_eq!(h[3].text, "H4");
        assert_eq!(h[4].level, 5); assert_eq!(h[4].text, "H5");
        assert_eq!(h[5].level, 6); assert_eq!(h[5].text, "H6");
    }

    #[test]
    fn test_parse_md_headings_no_space_after_hash() {
        let h = Notes::parse_md_headings("#invalid\n# valid");
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].text, "valid");
    }

    #[test]
    fn test_parse_md_empty() {
        let e = Notes::parse_md("");
        assert!(e.is_empty());
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
    fn test_update_scroll_target_no_data() {
        let mut n = Notes::new();
        n.update_scroll_target(5, 10);
        assert_eq!(n.scroll_ratio, 0.0);
    }

    #[test]
    fn test_update_scroll_target_proportional() {
        let mut n = Notes::new();
        n.line_count = 100;
        n.content = Some("lines\n".repeat(100));
        n.update_scroll_target(5, 10);
        assert!((n.scroll_ratio - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_update_scroll_target_first_page() {
        let mut n = Notes::new();
        n.line_count = 100;
        n.content = Some("x".to_string());
        n.update_scroll_target(0, 10);
        assert_eq!(n.scroll_ratio, 0.0);
    }

    #[test]
    fn test_update_scroll_target_last_page() {
        let mut n = Notes::new();
        n.line_count = 100;
        n.content = Some("x".to_string());
        n.update_scroll_target(9, 10);
        assert!((n.scroll_ratio - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_update_scroll_target_heading_sync() {
        let mut n = Notes::new();
        n.line_count = 100;
        n.typ_line_count = 100;
        n.content = Some("x".to_string());
        n.typ_headings = vec![
            HeadingInfo { level: 1, text: "A".into(), line: 0 },
            HeadingInfo { level: 2, text: "B".into(), line: 30 },
            HeadingInfo { level: 2, text: "C".into(), line: 50 },
            HeadingInfo { level: 2, text: "D".into(), line: 80 },
        ];
        n.md_headings = vec![
            HeadingInfo { level: 1, text: "A".into(), line: 0 },
            HeadingInfo { level: 2, text: "B".into(), line: 10 },
            HeadingInfo { level: 2, text: "C".into(), line: 20 },
            HeadingInfo { level: 2, text: "D".into(), line: 30 },
        ];

        n.update_scroll_target(0, 100);
        assert_eq!(n.scroll_ratio, 0.0);

        n.update_scroll_target(1, 100);
        assert!((n.scroll_ratio - 0.0).abs() < 0.01);

        n.update_scroll_target(34, 100);
        assert!((n.scroll_ratio - 0.333).abs() < 0.01);

        n.update_scroll_target(67, 100);
        assert!((n.scroll_ratio - 0.667).abs() < 0.01);

        n.update_scroll_target(100, 100);
        assert!((n.scroll_ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_update_scroll_target_no_md_headings() {
        let mut n = Notes::new();
        n.line_count = 50;
        n.typ_line_count = 50;
        n.content = Some("x".to_string());
        n.typ_headings = vec![HeadingInfo { level: 1, text: "A".into(), line: 0 }];

        n.update_scroll_target(3, 10);
        assert!((n.scroll_ratio - 0.3).abs() < 0.01);
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
    fn test_load_missing_file() {
        let n = Notes::load("/tmp/_nonexistent_12345.typ", "");
        assert!(n.content.is_none());
        assert!(n.elements.is_empty());
        assert_eq!(n.line_count, 0);
    }

    #[test]
    fn test_new_is_empty() {
        let n = Notes::new();
        assert!(n.path.is_none());
        assert!(n.content.is_none());
        assert!(n.typ_headings.is_empty());
        assert!(n.elements.is_empty());
    }

    #[test]
    fn test_parse_md_empty_string_after_whitespace() {
        let e1 = Notes::parse_md("   \n\n  ");
        let e2 = Notes::parse_md("");
        assert!(e1.is_empty(), "whitespace-only should yield no elements");
        assert!(e2.is_empty());
    }

    #[test]
    fn test_parse_typ_headings_inline_equals() {
        let h = Notes::parse_typ_headings("not=heading\n=== valid");
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].text, "valid");
    }

    #[test]
    fn test_parse_md_headings_double_hash_no_space() {
        let h = Notes::parse_md_headings("##valid");
        assert!(h.is_empty());
    }
}