use crate::notes::{MdElement, Notes};
use egui::{Color32, RichText, Ui};

pub fn draw(notes: &Notes, ui: &mut Ui) {
    if notes.elements.is_empty() {
        ui.colored_label(Color32::GRAY, "Las notas están vacías");
        return;
    }

    for element in &notes.elements {
        match element {
            MdElement::Heading(level, text) => {
                let size = match level {
                    1 => 22.0,
                    2 => 18.0,
                    3 => 16.0,
                    4 => 14.0,
                    5 => 13.0,
                    _ => 12.0,
                };
                ui.add_space(8.0);
                ui.label(RichText::new(text).size(size).strong());
                ui.add_space(4.0);
            }
            MdElement::Paragraph(text) => {
                ui.label(format_inline(text));
                ui.add_space(4.0);
            }
            MdElement::ListItem(text) => {
                ui.label(format!("  •  {}", format_inline(text).text()));
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

#[cfg(test)]
mod tests {
    use super::format_inline;

    #[test]
    fn test_format_inline_plain() {
        let rt = format_inline("hello world");
        assert_eq!(rt.text(), "hello world");
    }

    #[test]
    fn test_format_inline_bold() {
        let rt = format_inline("**bold text**");
        assert!(!rt.text().contains("**"));
        assert!(rt.text().contains("bold text"));
    }

    #[test]
    fn test_format_inline_code() {
        let rt = format_inline("use `fn()` here");
        assert!(rt.text().contains("fn()"));
        assert!(!rt.text().contains("`"));
    }

    #[test]
    fn test_format_inline_bold_and_code() {
        let rt = format_inline("**`x`**");
        assert_eq!(rt.text(), "x");
    }
}
