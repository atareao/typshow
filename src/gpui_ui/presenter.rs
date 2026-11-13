//! Ventana del presentador: página actual + siguiente, navegación, tema y notas.

use std::borrow::Cow;
use std::ops::Range;

use gpui::{
    App, AppContext, Bounds, Context, CursorStyle, Element, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, GlobalElementId, Hsla, InspectorElementId, InteractiveElement,
    KeyDownEvent, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ObjectFit,
    PaintQuad, Pixels, Point, Render, ShapedLine, SharedString, Style, Subscription, TextAlign,
    TextRun, UTF16Selection, UnderlineStyle, Window, WindowBounds, WindowHandle, WindowOptions,
    div, fill, img, point, prelude::*, px, relative, size,
};

use super::audience::AudienceView;
use super::deck::Deck;
use super::edit::TextEdit;
use super::keys::{apply_nav, clamp_boundary};
use super::theme::{
    BORDER_WIDTH, CARET_WIDTH, CONTROL_HEIGHT, CONTROL_PADDING_X, FONT_SIZE_BODY,
    FONT_SIZE_CAPTION, FONT_SIZE_TITLE, NOTES_PANEL_WIDTH, PANE_PADDING, Palette, RADIUS_BUTTON,
    RADIUS_CARD, SPACING_MD, SPACING_SM, SPACING_XXS, WINDOW_PADDING, palette,
};
use crate::notes::Notes;

/// Ejecuta `action` sobre el `Deck` compartido y notifica a los observers.
fn run<C: AppContext>(deck: &Entity<Deck>, action: fn(&mut Deck), cx: &mut C) {
    deck.update(cx, |deck, cx| {
        action(deck);
        cx.notify();
    });
}

// ---------------------------------------------------------------------------
// Modelo de edición: lógica pura, sin GPUI (testeable en `cargo test`).
//
// Tarea 3.12 (M4). El puente con la plataforma está implementado
// (`impl EntityInputHandler for PresenterView`), traduce UTF-16 ↔ bytes y
// vuelca cada cambio a los buffers de `Notes`; el `TextElement` pinta el texto
// (multilínea, resaltado, subrayado del IME y cursor) y rellena
// `note_layout`/`heading_layout`, que consumen el hit-testing del IME y los
// handlers de ratón (clic, arrastre y Shift+clic, tarea 3.12 (c)).
// ---------------------------------------------------------------------------

/// Editor activo de la ventana de presentador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorTarget {
    /// Cuerpo de la nota (multilínea).
    Note,
    /// Título de la nota (una sola línea).
    Heading,
}

/// Comando derivado de una pulsación para el editor activo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorCommand {
    /// Cancela la edición (Esc).
    Cancel,
    /// Confirma y guarda (Enter en el título).
    Save,
    /// Inserta un salto de línea (Enter en el cuerpo).
    Newline,
    /// Borra el carácter anterior.
    Backspace,
    /// Borra el carácter siguiente.
    Delete,
    MoveLeft,
    MoveRight,
    MoveHome,
    MoveEnd,
    SelectLeft,
    SelectRight,
    /// Extiende la selección hasta el inicio (Shift+Home).
    SelectToStart,
    /// Extiende la selección hasta el final (Shift+End).
    SelectToEnd,
    /// Selecciona todo (Ctrl+A).
    SelectAll,
    Copy,
    Cut,
    Paste,
    /// Carácter imprimible: lo entrega la plataforma al `EntityInputHandler`.
    PlatformText,
    /// Tecla sin efecto en el editor.
    Ignore,
}

/// Traduce una tecla a un comando del editor.
///
/// Decisión (a): en Linux (Wayland y X11) un carácter imprimible **no** se
/// inserta aquí. Tras los `on_key_down`, el backend entrega `key_char` al
/// input handler con foco con `replace_text_in_range(None, key_char)`
/// (`gpui_linux/src/linux/wayland/window.rs:1596`,
/// `gpui_linux/src/linux/x11/window.rs:1208`), de modo que devolver
/// [`EditorCommand::PlatformText`] evita la doble inserción. Solo se llega a esa
/// entrega si el evento no se consume (`DispatchEventResult::propagate`,
/// `gpui/src/window.rs:6037`), así que el resto de teclas que sí gestionamos
/// llaman a `cx.stop_propagation()`.
fn editor_command(
    target: EditorTarget,
    key: &str,
    key_char: Option<&str>,
    shift: bool,
    control: bool,
) -> EditorCommand {
    if control {
        return match key {
            "a" => EditorCommand::SelectAll,
            "c" => EditorCommand::Copy,
            "x" => EditorCommand::Cut,
            "v" => EditorCommand::Paste,
            _ => EditorCommand::Ignore,
        };
    }
    match key {
        "escape" => EditorCommand::Cancel,
        "backspace" => EditorCommand::Backspace,
        "delete" => EditorCommand::Delete,
        "enter" => match target {
            EditorTarget::Note => EditorCommand::Newline,
            EditorTarget::Heading => EditorCommand::Save,
        },
        "left" => match shift {
            true => EditorCommand::SelectLeft,
            false => EditorCommand::MoveLeft,
        },
        "right" => match shift {
            true => EditorCommand::SelectRight,
            false => EditorCommand::MoveRight,
        },
        "home" => match shift {
            true => EditorCommand::SelectToStart,
            false => EditorCommand::MoveHome,
        },
        "end" => match shift {
            true => EditorCommand::SelectToEnd,
            false => EditorCommand::MoveEnd,
        },
        _ => match key_char {
            Some(ch) if !ch.is_empty() => EditorCommand::PlatformText,
            _ => EditorCommand::Ignore,
        },
    }
}

/// `true` si el comando puede cambiar el texto (no solo la selección).
fn command_mutates_text(command: EditorCommand) -> bool {
    matches!(
        command,
        EditorCommand::Newline | EditorCommand::Backspace | EditorCommand::Delete
    )
}

/// Aplica a `edit` los comandos que solo afectan al modelo de texto.
///
/// Devuelve `true` si el comando lo gestiona el modelo (aunque no cambie nada).
fn apply_model_command(edit: &mut TextEdit, command: EditorCommand) -> bool {
    match command {
        EditorCommand::Newline => {
            edit.replace_selection("\n");
            true
        }
        EditorCommand::Backspace => {
            edit.backspace();
            true
        }
        EditorCommand::Delete => {
            edit.delete();
            true
        }
        EditorCommand::MoveLeft => {
            edit.move_left();
            true
        }
        EditorCommand::MoveRight => {
            edit.move_right();
            true
        }
        EditorCommand::MoveHome => {
            edit.move_home();
            true
        }
        EditorCommand::MoveEnd => {
            edit.move_end();
            true
        }
        EditorCommand::SelectLeft => {
            edit.select_left();
            true
        }
        EditorCommand::SelectRight => {
            edit.select_right();
            true
        }
        EditorCommand::SelectToStart => {
            edit.select_to(0);
            true
        }
        EditorCommand::SelectToEnd => {
            let end = edit.text.len();
            edit.select_to(end);
            true
        }
        EditorCommand::SelectAll => {
            edit.select_all();
            true
        }
        EditorCommand::Cancel
        | EditorCommand::Save
        | EditorCommand::Copy
        | EditorCommand::Cut
        | EditorCommand::Paste
        | EditorCommand::PlatformText
        | EditorCommand::Ignore => false,
    }
}

/// Sustituye texto con un rango en UTF-16, como `EntityInputHandler::replace_text_in_range`.
///
/// En el título (`single_line`) los saltos de línea (`\n` y `\r`) se convierten
/// en espacios, igual que hace la plantilla de GPUI al pegar. Fuera de ese caso
/// el texto se toma prestado (`Cow::Borrowed`): escribir una tecla no debe
/// copiar la cadena.
fn replace_text_from_utf16(
    edit: &mut TextEdit,
    range_utf16: Option<Range<usize>>,
    new_text: &str,
    single_line: bool,
) {
    let new_text = flatten_single_line(new_text, single_line);
    match range_utf16 {
        Some(range) => {
            let bytes = edit.range_from_utf16(&range);
            edit.replace_range(bytes, &new_text);
        }
        None => edit.replace_selection(&new_text),
    }
}

/// Aplana los saltos de línea a espacios cuando el editor es de una sola línea.
///
/// Presta el texto si no hay nada que aplanar (`Cow::Borrowed`) para no copiar
/// en el camino caliente de cada tecla.
fn flatten_single_line(text: &str, single_line: bool) -> Cow<'_, str> {
    if single_line {
        Cow::Owned(text.replace(['\r', '\n'], " "))
    } else {
        Cow::Borrowed(text)
    }
}

/// Selección actual en unidades UTF-16, como la espera GPUI.
fn selected_range_utf16(edit: &TextEdit) -> Range<usize> {
    edit.range_to_utf16(&edit.selection)
}

/// Fija la selección a partir de un rango en UTF-16 (`set_selected_text_range`).
fn set_selection_from_utf16(edit: &mut TextEdit, range_utf16: &Range<usize>) {
    let bytes = edit.range_from_utf16(range_utf16);
    edit.set_cursor(bytes.start);
    edit.select_to(bytes.end);
}

/// Vuelca el estado del editor en los buffers de `Notes`, que siguen siendo la
/// fuente de verdad compartida con la app egui.
///
/// `dirty` solo se marca cuando el texto de la nota cambió **de verdad**:
/// mover el cursor, cambiar la selección o desmarcar una composición IME no
/// debe disparar el autoguardado. El cursor se propaga siempre. Devuelve `true`
/// si el texto cambió.
///
/// El texto entra por `&str` y se reescribe el buffer sin reasignar cuando hay
/// capacidad (`clear` + `push_str`), evitando el `clone` por tecla.
fn write_back(notes: &mut Notes, target: EditorTarget, text: &str, cursor: usize) -> bool {
    match target {
        EditorTarget::Note => {
            notes.cursor = cursor;
            if notes.edit_buffer != text {
                notes.edit_buffer.clear();
                notes.edit_buffer.push_str(text);
                notes.dirty = true;
                true
            } else {
                false
            }
        }
        EditorTarget::Heading => {
            notes.heading_cursor = cursor;
            if notes.heading_edit_buffer != text {
                notes.heading_edit_buffer.clear();
                notes.heading_edit_buffer.push_str(text);
            }
            false
        }
    }
}

/// Offset en bytes del inicio de cada línea (incluye la línea vacía final).
fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

/// Rango en bytes de cada línea, sin el `\n` que la termina.
fn line_ranges(text: &str) -> Vec<Range<usize>> {
    let starts = line_starts(text);
    let mut ranges = Vec::with_capacity(starts.len());
    for (index, start) in starts.iter().enumerate() {
        let end = starts.get(index + 1).map_or(text.len(), |next| next - 1);
        ranges.push(*start..end.max(*start));
    }
    ranges
}

/// Índice de la línea que contiene el offset `byte`.
fn line_index_for_byte(starts: &[usize], byte: usize) -> usize {
    starts.iter().rposition(|start| *start <= byte).unwrap_or(0)
}

/// Índice de línea para un desplazamiento vertical, acotado a las líneas que hay.
fn line_index_for_y(local_y: Pixels, line_height: Pixels, line_count: usize) -> usize {
    if line_count == 0 {
        return 0;
    }
    let height = f32::from(line_height).max(1.);
    ((f32::from(local_y).max(0.) / height) as usize).min(line_count - 1)
}

/// Subcadena de `text` en `range`, acotada al texto y a fronteras de carácter.
fn clamped_slice<'a>(text: &'a str, range: &Range<usize>) -> &'a str {
    let start = clamp_boundary(text, range.start);
    let end = clamp_boundary(text, range.end).max(start);
    &text[start..end]
}

/// Intersección no vacía de dos rangos de bytes.
fn intersect(a: &Range<usize>, b: &Range<usize>) -> Option<Range<usize>> {
    let start = a.start.max(b.start);
    let end = a.end.min(b.end);
    (start < end).then_some(start..end)
}

/// Tramos de una línea, separando texto normal, seleccionado y marcado por el IME.
///
/// Las líneas vacías se conforman con un espacio (un run de longitud 1) porque
/// `shape_line` no tiene sentido con texto vacío; los índices que se le piden
/// después van siempre acotados al texto real del editor.
fn line_runs(
    base: &TextRun,
    range: &Range<usize>,
    selection: &Range<usize>,
    marked: Option<&Range<usize>>,
    selection_bg: Hsla,
    marked_color: Hsla,
) -> Vec<TextRun> {
    if range.is_empty() {
        return vec![TextRun {
            len: 1,
            ..base.clone()
        }];
    }

    let mut cuts = vec![range.start, range.end];
    for byte in [selection.start, selection.end] {
        if byte > range.start && byte < range.end {
            cuts.push(byte);
        }
    }
    if let Some(marked) = marked {
        for byte in [marked.start, marked.end] {
            if byte > range.start && byte < range.end {
                cuts.push(byte);
            }
        }
    }
    cuts.sort_unstable();
    cuts.dedup();

    cuts.windows(2)
        .map(|pair| {
            let (start, end) = (pair[0], pair[1]);
            let mut run = base.clone();
            run.len = end - start;
            if start >= selection.start && end <= selection.end && !selection.is_empty() {
                run.background_color = Some(selection_bg);
            }
            if let Some(marked) = marked
                && start >= marked.start
                && end <= marked.end
            {
                run.underline = Some(UnderlineStyle {
                    color: Some(marked_color),
                    thickness: px(BORDER_WIDTH),
                    wavy: false,
                });
            }
            run
        })
        .collect()
}

/// Layout conformado de un editor durante el último frame.
///
/// Lo escribe el `TextElement` en cada `paint` (se guarda en `note_layout` o
/// `heading_layout`) y lo leen `bounds_for_range`,
/// `selection_bounds_for_range` y `character_index_for_point` (necesarios para
/// el IME: candidatos, cursor de composición y hit-test con el ratón) sin
/// volver a conformar el texto.
struct EditorLayout {
    /// Cada línea: rango en bytes del texto real + línea conformada. La línea
    /// conformada puede ser un espacio cuando la línea está vacía.
    lines: Vec<(Range<usize>, ShapedLine)>,
    /// Rectángulo del editor durante el último frame.
    bounds: Bounds<Pixels>,
    /// Altura de línea usada al conformar.
    line_height: Pixels,
    /// Texto exacto que se conformó (para descartar el layout si cambió).
    text: SharedString,
}

impl EditorLayout {
    /// Índice de línea que contiene el offset `byte`.
    fn line_index_for_byte(&self, byte: usize) -> usize {
        self.lines
            .iter()
            .rposition(|(range, _)| range.start <= byte)
            .unwrap_or(0)
    }

    /// Offset en bytes correspondiente a un punto, o `None` si el punto queda
    /// fuera del rectángulo del editor (`localize` falla), igual que la
    /// plantilla de GPUI: ahí no hay posición que devolver.
    fn index_for_point(&self, position: Point<Pixels>) -> Option<usize> {
        let local = self.bounds.localize(&position)?;
        if self.lines.is_empty() {
            return Some(0);
        }
        let line_index = line_index_for_y(local.y, self.line_height, self.lines.len());
        let (range, line) = &self.lines[line_index];
        let in_line = line
            .closest_index_for_x(local.x)
            .min(range.end - range.start);
        Some(range.start + in_line)
    }

    /// Rectángulo que cubre el rango `range` (en bytes), acotado a la primera
    /// línea que lo contiene. `origin` es la esquina superior izquierda del
    /// editor en coordenadas de ventana.
    fn bounds_for_byte_range(
        &self,
        range: &Range<usize>,
        origin: Point<Pixels>,
    ) -> Option<Bounds<Pixels>> {
        let line_index = self.line_index_for_byte(range.start);
        let (line_range, line) = self.lines.get(line_index)?;
        let start = range.start.clamp(line_range.start, line_range.end) - line_range.start;
        let end = range.end.clamp(line_range.start, line_range.end) - line_range.start;
        let top = origin.y + self.line_height * line_index as f32;
        let bottom = top + self.line_height;
        Some(Bounds::from_corners(
            point(origin.x + line.x_for_index(start), top),
            point(origin.x + line.x_for_index(end), bottom),
        ))
    }

    /// Rectángulos de selección (uno por línea) para el rango `range`.
    fn selection_bounds_for_byte_range(
        &self,
        range: &Range<usize>,
        origin: Point<Pixels>,
    ) -> Vec<Bounds<Pixels>> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(index, (line_range, line))| {
                let overlap = intersect(line_range, range)?;
                let top = origin.y + self.line_height * index as f32;
                let bottom = top + self.line_height;
                Some(Bounds::from_corners(
                    point(
                        origin.x + line.x_for_index(overlap.start - line_range.start),
                        top,
                    ),
                    point(
                        origin.x + line.x_for_index(overlap.end - line_range.start),
                        bottom,
                    ),
                ))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// `TextElement`: pintado del editor y puente de entrada con la plataforma.
// ---------------------------------------------------------------------------

/// Elemento que pinta y registra un editor de texto del presentador.
///
/// Renderiza el buffer del modelo como texto multilínea (`\n` separa líneas),
/// con el resaltado de la selección, el subrayado del rango marcado por el IME
/// y el cursor; al final del `paint` guarda el `EditorLayout` recién conformado
/// en la vista, que es lo que consultan la geometría del IME y el hit-testing.
struct TextElement {
    view: Entity<PresenterView>,
    target: EditorTarget,
}

/// Estado que el `TextElement` calcula en `prepaint` y consume en `paint`.
#[derive(Default)]
struct EditorFrame {
    /// Layout conformado del frame; `None` si el editor no tiene modelo.
    layout: Option<EditorLayout>,
    /// Un rectángulo por línea solapada con la selección.
    selection: Vec<PaintQuad>,
    /// Subrayado del rango marcado por el IME (pre-edit).
    marked: Vec<PaintQuad>,
    /// Cursor, solo cuando la selección está colapsada.
    caret: Option<PaintQuad>,
}

/// Opacidad del resaltado de selección (acento del tema sobre la superficie).
const SELECTION_ALPHA: f32 = 0.3;

/// Conforma el texto del editor y calcula la geometría que pinta el `TextElement`.
///
/// Es una función libre, no un método, para poder ejercitarla en tests headless
/// con el `TestPlatform`: separa el cálculo (líneas, resaltado, subrayado y
/// cursor) de la lectura del modelo y del guardado en la vista.
fn editor_frame(
    text: &str,
    selection: &Range<usize>,
    marked: Option<&Range<usize>>,
    cursor: usize,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    p: &Palette,
) -> EditorFrame {
    let style = window.text_style();
    let line_height = window.line_height();
    let font_size = style.font_size.to_pixels(window.rem_size());
    let text_color: Hsla = p.fg.into();
    let selection_color = Hsla::from(p.accent).opacity(SELECTION_ALPHA);
    let marked_color: Hsla = p.accent.into();
    let base = TextRun {
        len: 0,
        font: style.font(),
        color: text_color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };

    let ranges = line_ranges(text);
    let mut lines = Vec::with_capacity(ranges.len());
    for range in &ranges {
        // Una línea vacía se conforma con un espacio (un run de longitud 1), que
        // es lo que espera `line_runs`; los índices que se le piden después van
        // acotados al texto real y nunca pasan de 0 en esa línea.
        let slice = if range.is_empty() {
            " "
        } else {
            &text[range.clone()]
        };
        let runs = line_runs(
            &base,
            range,
            selection,
            marked,
            selection_color,
            marked_color,
        );
        let line = window
            .text_system()
            .shape_line(slice.into(), font_size, &runs, None);
        lines.push((range.clone(), line));
    }

    let layout = EditorLayout {
        lines,
        bounds,
        line_height,
        text: text.into(),
    };

    let selection_quads = if selection.is_empty() {
        Vec::new()
    } else {
        layout
            .selection_bounds_for_byte_range(selection, bounds.origin)
            .into_iter()
            .map(|bounds| fill(bounds, selection_color))
            .collect()
    };

    // El subrayado del pre-edit se pinta como una barra de 1px al pie del
    // rectángulo de cada línea tocada por el rango marcado.
    let marked_quads = marked
        .into_iter()
        .flat_map(|marked| layout.selection_bounds_for_byte_range(marked, bounds.origin))
        .map(|bounds| {
            let underline = Bounds::new(
                point(bounds.left(), bounds.bottom() - px(BORDER_WIDTH)),
                size(bounds.size.width, px(BORDER_WIDTH)),
            );
            fill(underline, marked_color)
        })
        .collect();

    let caret = if selection.is_empty() {
        let index = line_index_for_byte(&line_starts(text), cursor);
        layout.lines.get(index).map(|(range, line)| {
            let in_line = cursor.clamp(range.start, range.end) - range.start;
            let top = bounds.origin.y + line_height * index as f32;
            fill(
                Bounds::new(
                    point(bounds.left() + line.x_for_index(in_line), top),
                    size(px(CARET_WIDTH), line_height),
                ),
                text_color,
            )
        })
    } else {
        None
    };

    EditorFrame {
        layout: Some(layout),
        selection: selection_quads,
        marked: marked_quads,
        caret,
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = EditorFrame;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let line_count = self
            .view
            .read(cx)
            .editor(self.target)
            .map_or(1, |edit| line_ranges(&edit.text).len());
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (window.line_height() * line_count as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> EditorFrame {
        let p = {
            let view = self.view.read(cx);
            palette(view.deck.read(cx).dark)
        };
        // El `Ref` vive solo mientras dura el préstamo del modelo: `editor_frame`
        // recibe referencias y no necesita propiedad, así que no se clona por frame.
        let view = self.view.read(cx);
        let Some(edit) = view.editor(self.target) else {
            return EditorFrame::default();
        };
        editor_frame(
            &edit.text,
            &edit.selection,
            edit.marked.as_ref(),
            edit.cursor(),
            bounds,
            window,
            &p,
        )
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        frame: &mut EditorFrame,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.view.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );

        // Primero el resaltado (va detrás del texto) y después las líneas.
        for quad in frame.selection.drain(..) {
            window.paint_quad(quad);
        }
        for quad in frame.marked.drain(..) {
            window.paint_quad(quad);
        }
        if let Some(layout) = frame.layout.as_ref() {
            let line_height = layout.line_height;
            for (index, (_, line)) in layout.lines.iter().enumerate() {
                let origin = point(
                    bounds.origin.x,
                    bounds.origin.y + line_height * index as f32,
                );
                if let Err(error) =
                    line.paint(origin, line_height, TextAlign::Left, None, window, cx)
                {
                    tracing::warn!("no se pudo pintar la línea {index} del editor: {error}");
                }
            }
        }
        if focus_handle.is_focused(window)
            && let Some(caret) = frame.caret.take()
        {
            window.paint_quad(caret);
        }

        // El layout recién conformado se guarda para el frame siguiente: es lo
        // que resuelve la geometría del IME y el hit-testing del ratón.
        if let Some(layout) = frame.layout.take() {
            let target = self.target;
            self.view.update(cx, move |view, _cx| match target {
                EditorTarget::Note => view.note_layout = Some(layout),
                EditorTarget::Heading => view.heading_layout = Some(layout),
            });
        }
    }
}

/// Geometría y tipografía comunes a cualquier control Adwaita.
///
/// `id` es obligatorio porque el estado hover de GPUI se guarda por elemento.
fn control_base(id: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .h(px(CONTROL_HEIGHT))
        .px(px(CONTROL_PADDING_X))
        .rounded(px(RADIUS_BUTTON))
        .text_size(px(FONT_SIZE_BODY))
}

/// Control neutro: superficie con borde, como los botones normales de Adwaita.
fn control_neutral(id: &'static str, p: &Palette) -> gpui::Stateful<gpui::Div> {
    control_base(id)
        .bg(p.surface)
        .border(px(BORDER_WIDTH))
        .border_color(p.border)
        .text_color(p.fg)
        .hover(move |s| s.bg(p.hover).border_color(p.border_strong))
}

/// Control destructivo: botón neutro con texto de peligro.
fn control_destructive(id: &'static str, p: &Palette) -> gpui::Stateful<gpui::Div> {
    control_base(id)
        .bg(p.surface)
        .border(px(BORDER_WIDTH))
        .border_color(p.border)
        .text_color(p.danger)
        .hover(move |s| s.bg(p.hover).border_color(p.border_strong))
}

/// Control sugerido: relleno de acento con texto sobre el acento.
fn control_filled(id: &'static str, p: &Palette) -> gpui::Stateful<gpui::Div> {
    control_base(id)
        .bg(p.accent)
        .text_color(p.accent_fg)
        .hover(move |s| s.bg(p.accent_hover))
}

/// Control deshabilitado: sin hover ni interacción.
fn control_disabled(id: &'static str, p: &Palette) -> gpui::Stateful<gpui::Div> {
    control_base(id)
        .bg(p.disabled_bg)
        .border(px(BORDER_WIDTH))
        .border_color(p.border)
        .text_color(p.disabled_fg)
}

/// Botón de acción principal (relleno de acento).
fn action_button(
    deck: &Entity<Deck>,
    id: &'static str,
    label: &str,
    p: &Palette,
    action: fn(&mut Deck),
) -> impl IntoElement {
    let deck = deck.clone();
    control_filled(id, p)
        .child(label.to_string())
        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
            run(&deck, action, cx);
        })
}

/// Botón de navegación con estado habilitado/deshabilitado.
fn nav_button(
    deck: &Entity<Deck>,
    id: &'static str,
    label: &str,
    enabled: bool,
    p: &Palette,
    action: fn(&mut Deck),
) -> gpui::AnyElement {
    if enabled {
        let deck = deck.clone();
        control_neutral(id, p)
            .child(label.to_string())
            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                run(&deck, action, cx);
            })
            .into_any_element()
    } else {
        control_disabled(id, p)
            .child(label.to_string())
            .into_any_element()
    }
}

/// Cuelga de `element` los handlers de selección con el ratón del editor
/// `target`, cuyo contenedor envuelve al `TextElement`.
///
/// Clic (coloca el cursor, o extiende con Shift), arrastre (extiende) y final
/// del arrastre dentro o fuera del contenedor (`mouse_up` / `mouse_up_out`).
/// La geometría y el estado los resuelve la vista en
/// `begin_selection`/`extend_selection`; el estilo del cursor lo pone quien
/// llama, junto al resto del estilo del contenedor.
fn with_selection_handlers<E: InteractiveElement>(
    element: E,
    target: EditorTarget,
    cx: &mut Context<PresenterView>,
) -> E {
    element
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                this.begin_selection(target, event.position, event.modifiers.shift, window, cx);
            }),
        )
        .on_mouse_move(
            cx.listener(move |this, event: &MouseMoveEvent, window, cx| {
                // Un `mouse_move` sin el botón pulsado es un simple hover: no
                // debe extender nada aunque `is_selecting` quedara obsoleto.
                if event.pressed_button == Some(MouseButton::Left) {
                    this.extend_selection(target, event.position, window, cx);
                }
            }),
        )
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _event: &MouseUpEvent, _window, _cx| {
                this.end_selection();
            }),
        )
        .on_mouse_up_out(
            MouseButton::Left,
            cx.listener(|this, _event: &MouseUpEvent, _window, _cx| {
                this.end_selection();
            }),
        )
}

/// Vista de la ventana del presentador.
///
/// `deck`, `focus_handle` y los campos del editor (`note_edit`, `heading_edit`,
/// `*_layout`) los consume el puente `EntityInputHandler`; `is_selecting` marca
/// el arrastre de selección con el ratón en curso.
pub struct PresenterView {
    deck: Entity<Deck>,
    focus_handle: FocusHandle,
    audience: Option<WindowHandle<AudienceView>>,
    /// Hay un diálogo de fichero abierto (guard de reentrada).
    dialog_open: bool,
    /// Modelo del editor del cuerpo de la nota, activo mientras se edita.
    note_edit: Option<TextEdit>,
    /// Modelo del editor del título, activo mientras se edita.
    heading_edit: Option<TextEdit>,
    /// Último layout del editor de notas (hit-testing e IME).
    note_layout: Option<EditorLayout>,
    /// Último layout del editor del título (hit-testing e IME).
    heading_layout: Option<EditorLayout>,
    /// Arrastre de selección con el ratón en curso.
    is_selecting: bool,
    _sub: Subscription,
    _close_sub: Subscription,
}

impl PresenterView {
    /// Crea la vista del presentador para la `deck` compartida.
    pub fn new(deck: Entity<Deck>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        let _sub = cx.observe(&deck, |_this, _deck, cx| cx.notify());

        // Mantiene `audience`/`projection` sincronizados cuando la ventana de
        // audiencia se cierra (Esc, botón del compositor, etc.).
        let this = cx.entity().downgrade();
        let _close_sub = cx.on_window_closed(move |cx, window_id| {
            this.update(cx, |view, cx| {
                if view.audience.map(|h| h.window_id()) == Some(window_id) {
                    view.audience = None;
                    view.deck.update(cx, |deck, cx| {
                        deck.projection = false;
                        cx.notify();
                    });
                    cx.notify();
                }
            })
            .ok();
        });

        let mut view = Self {
            deck,
            focus_handle,
            audience: None,
            dialog_open: false,
            note_edit: None,
            heading_edit: None,
            note_layout: None,
            heading_layout: None,
            is_selecting: false,
            _sub,
            _close_sub,
        };
        if view.deck.read(cx).projection {
            view.open_audience(cx);
        }
        view
    }

    /// Abre la ventana de audiencia (fullscreen) y guarda su handle.
    fn open_audience(&mut self, cx: &mut Context<Self>) {
        let deck = self.deck.clone();
        // Preferimos un monitor secundario (no el primario) para la audiencia;
        // si solo hay uno, el primario. En Niri/Wayland la ventana se abre en el
        // display indicado y luego pasa a fullscreen en él.
        let primary = cx.primary_display().map(|d| d.id());
        let display_id = cx
            .displays()
            .into_iter()
            .find(|d| Some(d.id()) != primary)
            .or_else(|| cx.primary_display())
            .map(|d| d.id());
        self.audience = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Fullscreen(Bounds::default())),
                    // `display_id` solo lo respeta el backend X11; en Wayland
                    // (Niri) los toplevel lo ignoran. Para fijar la salida hay
                    // que usar una regla de compositor sobre `app_id`.
                    display_id,
                    app_id: Some("com.atareao.typshow.audience".to_string()),
                    ..Default::default()
                },
                move |window, cx| cx.new(|cx| AudienceView::new(deck, window, cx)),
            )
            .ok();
    }

    /// Cierra la ventana de audiencia si está abierta.
    fn close_audience(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.audience.take() {
            let _ = handle.update(cx, |_view, window, _cx| window.remove_window());
        }
    }

    /// Muestra u oculta la proyección, reconciliando el handle con el estado real.
    fn toggle_projection(&mut self, cx: &mut Context<Self>) {
        // `is_active` refleja el FOCO, no si la ventana sigue viva: comprobamos
        // pertenencia a `cx.windows()` para no dejar ventanas huérfanas ni duplicar.
        if let Some(handle) = self.audience {
            let id = handle.window_id();
            if cx.windows().iter().all(|w| w.window_id() != id) {
                self.audience = None;
            }
        }
        if self.audience.is_some() {
            self.close_audience(cx);
            self.deck.update(cx, |deck, cx| {
                deck.projection = false;
                cx.notify();
            });
        } else {
            self.open_audience(cx);
            self.deck.update(cx, |deck, cx| {
                deck.projection = true;
                cx.notify();
            });
        }
    }

    /// Abre el diálogo de fichero y carga el `.typ` elegido.
    ///
    /// El diálogo `rfd` es asíncrono y se espera en el executor de fondo, así que
    /// el hilo de UI nunca se bloquea. `dialog_open` evita reentradas mientras
    /// haya un diálogo en curso.
    fn open_file_dialog(&mut self, cx: &mut Context<Self>) {
        if self.dialog_open {
            return;
        }
        self.dialog_open = true;
        cx.notify();
        let deck = self.deck.clone();
        cx.spawn(async move |this, cx| {
            let picked = cx
                .background_spawn(async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Typst", &["typ"])
                        .set_title("Abrir presentación Typst")
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                })
                .await;
            this.update(cx, |this, cx| {
                this.dialog_open = false;
                if let Some(path) = picked {
                    let path = path.to_string_lossy().to_string();
                    deck.update(cx, |deck, cx| deck.open_file_async(&path, cx));
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Ejecuta `f` sobre el `Deck` desde un handler de la vista.
    fn with_deck(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Deck)) {
        self.deck.update(cx, |deck, cx| {
            f(deck);
            cx.notify();
        });
    }

    /// Panel lateral de notas (ver/editar/acciones).
    fn notes_panel(&self, cx: &mut Context<Self>, p: &Palette) -> impl IntoElement {
        let notes = &self.deck.read(cx).notes;
        let available = notes.has_content();
        let editing = notes.editing;
        let heading = notes.heading.clone();
        let content = notes.note_content.clone().unwrap_or_default();
        let heading_editing = notes.heading_editing;

        let body: gpui::AnyElement = if editing {
            div()
                .flex()
                .flex_col()
                .gap(px(SPACING_SM))
                .child(
                    with_selection_handlers(
                        div()
                            .flex_1()
                            .min_h_0()
                            .p(px(SPACING_MD))
                            .bg(p.surface)
                            .border(px(BORDER_WIDTH))
                            .border_color(p.border)
                            .rounded(px(RADIUS_CARD))
                            .id("notes-edit-body")
                            .overflow_y_scroll()
                            .text_color(p.fg)
                            .cursor(CursorStyle::IBeam),
                        EditorTarget::Note,
                        cx,
                    )
                    .child(TextElement {
                        view: cx.entity(),
                        target: EditorTarget::Note,
                    }),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(SPACING_SM))
                        .child(
                            control_filled("notes-guardar", p)
                                .child("Guardar")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _e, _w, cx| {
                                        this.flush_editor(EditorTarget::Note, cx);
                                        this.with_deck(cx, |d| {
                                            if let Err(e) = d.notes.save_current() {
                                                d.error = Some(e.to_string());
                                            }
                                        })
                                    }),
                                ),
                        )
                        .child(
                            control_neutral("notes-cancelar", p)
                                .child("Cancelar")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _e, _w, cx| {
                                        this.with_deck(cx, |d| d.notes.cancel_edit())
                                    }),
                                ),
                        ),
                )
                .into_any_element()
        } else if !available {
            div()
                .text_color(p.muted)
                .child("No hay base de datos de notas para este documento")
                .into_any_element()
        } else {
            let content = if content.trim().is_empty() {
                "Las notas están vacías".to_string()
            } else {
                content
            };
            div()
                .flex_1()
                .min_h_0()
                .p(px(SPACING_MD))
                .bg(p.surface)
                .border(px(BORDER_WIDTH))
                .border_color(p.border)
                .rounded(px(RADIUS_CARD))
                .id("notes-view-body")
                .overflow_y_scroll()
                .text_color(if content == "Las notas están vacías" {
                    p.muted
                } else {
                    p.fg
                })
                .child(content)
                .into_any_element()
        };

        let actions = div()
            .flex()
            .gap(px(SPACING_SM))
            .flex_wrap()
            .child(
                control_filled("notes-editar", p)
                    .child("Editar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| d.notes.start_edit())
                        }),
                    ),
            )
            .child(
                control_neutral("notes-nueva", p)
                    .child("Nueva")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| {
                                if let Err(e) = d.notes.insert_note() {
                                    d.error = Some(e.to_string());
                                }
                            })
                        }),
                    ),
            )
            .child(
                control_destructive("notes-eliminar", p)
                    .child("Eliminar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| {
                                if let Err(e) = d.notes.delete_current() {
                                    d.error = Some(e.to_string());
                                }
                            })
                        }),
                    ),
            )
            .child(
                control_neutral("notes-reimportar", p)
                    .child("Reimportar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| {
                                if let Err(e) = d.reimport_notes() {
                                    d.error = Some(e);
                                }
                            })
                        }),
                    ),
            )
            .child(
                control_neutral("notes-titulo", p)
                    .child("Título")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| d.notes.start_edit_heading())
                        }),
                    ),
            );

        div()
            .w(px(NOTES_PANEL_WIDTH))
            .h_full()
            .flex()
            .flex_col()
            .gap(px(SPACING_SM))
            .pl(px(SPACING_MD))
            .border_l(px(BORDER_WIDTH))
            .border_color(p.border)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .text_size(px(FONT_SIZE_TITLE))
                    .child(div().child("Notas")),
            )
            .children((available || heading_editing).then(|| {
                if heading_editing {
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .gap(px(SPACING_SM))
                        .items_center()
                        .child(
                            with_selection_handlers(
                                div()
                                    .flex()
                                    .items_center()
                                    .h(px(CONTROL_HEIGHT))
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .px(px(SPACING_SM))
                                    .bg(p.surface)
                                    .border(px(BORDER_WIDTH))
                                    .border_color(p.accent)
                                    .rounded(px(RADIUS_BUTTON))
                                    .text_color(p.fg)
                                    .cursor(CursorStyle::IBeam),
                                EditorTarget::Heading,
                                cx,
                            )
                            .child(TextElement {
                                view: cx.entity(),
                                target: EditorTarget::Heading,
                            }),
                        )
                        .child(
                            control_filled("notes-titulo-ok", p)
                                .flex_shrink_0()
                                .child("OK")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _e, _w, cx| {
                                        this.flush_editor(EditorTarget::Heading, cx);
                                        this.with_deck(cx, |d| {
                                            if let Err(e) = d.notes.save_heading() {
                                                d.error = Some(e.to_string());
                                            }
                                        })
                                    }),
                                ),
                        )
                        .child(
                            control_neutral("notes-titulo-cancelar", p)
                                .flex_shrink_0()
                                .child("X")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _e, _w, cx| {
                                        this.with_deck(cx, |d| d.notes.cancel_edit_heading())
                                    }),
                                ),
                        )
                } else {
                    div()
                        .w_full()
                        .overflow_hidden()
                        .text_size(px(FONT_SIZE_CAPTION))
                        .text_color(p.muted)
                        .child(
                            heading
                                .clone()
                                .unwrap_or_else(|| "(sin título)".to_string()),
                        )
                }
            }))
            .child(body)
            .children((available && !editing && !heading_editing).then_some(actions))
    }
}

impl Render for PresenterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Al empezar cada frame se crean (o descartan) los modelos de edición
        // según los flags de `Notes`: cubre todos los caminos que entran y salen
        // del modo edición (botones, «Nueva», guardar, cancelar, Esc).
        self.sync_editors(cx);
        let (page, total, dark, error, notes_visible, projection, compiling) = {
            let deck = self.deck.read(cx);
            (
                deck.page,
                deck.total,
                deck.dark,
                deck.error.clone(),
                deck.notes_visible,
                deck.projection,
                deck.loading,
            )
        };
        let (image, next_image, loading, next_loading) = self.deck.update(cx, |deck, cx| {
            let image = deck.current_image(cx);
            let next = deck.next_image(cx);
            let loading = image.is_none() && (deck.loading || deck.is_pending(deck.page));
            let next_loading = next.is_none()
                && (deck.loading || deck.next_index().is_some_and(|i| deck.is_pending(i)));
            (image, next, loading, next_loading)
        });
        let (has_prev, has_next) = {
            let deck = self.deck.read(cx);
            (deck.has_prev(), deck.has_next())
        };
        let (editing, heading_editing) = {
            let notes = &self.deck.read(cx).notes;
            (notes.editing, notes.heading_editing)
        };

        let p = palette(dark);
        let total_label = if total == 0 { 1 } else { total };
        let deck = self.deck.clone();

        let current_pane = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .px(px(WINDOW_PADDING))
                    .py(px(SPACING_XXS))
                    .text_size(px(FONT_SIZE_CAPTION))
                    .text_color(p.muted)
                    .child("Actual"),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .p(px(PANE_PADDING))
                    .child(match image {
                        Some(image) => img(image)
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .into_any_element(),
                        None => div()
                            .text_color(p.fg)
                            .child(if compiling {
                                "Compilando…".to_string()
                            } else if loading {
                                "Cargando…".to_string()
                            } else {
                                error.clone().unwrap_or_else(|| "Sin documento".to_string())
                            })
                            .into_any_element(),
                    }),
            );

        let next_pane = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .border_t(px(BORDER_WIDTH))
            .border_color(p.border)
            .child(
                div()
                    .px(px(WINDOW_PADDING))
                    .py(px(SPACING_XXS))
                    .text_size(px(FONT_SIZE_CAPTION))
                    .text_color(p.muted)
                    .child("Siguiente"),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .p(px(PANE_PADDING))
                    .child(match next_image {
                        Some(image) => img(image)
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .into_any_element(),
                        None => div()
                            .text_size(px(FONT_SIZE_CAPTION))
                            .text_color(p.muted)
                            .child(if compiling {
                                "Compilando…".to_string()
                            } else if next_loading {
                                "Cargando…".to_string()
                            } else if total == 0 {
                                String::new()
                            } else {
                                "Fin de la presentacion".to_string()
                            })
                            .into_any_element(),
                    }),
            );

        let page_area = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(p.view_bg)
            .rounded(px(RADIUS_CARD))
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, {
                let deck = deck.clone();
                move |_event, _window, cx| {
                    if !deck.read(cx).is_editing() {
                        run(&deck, Deck::prev, cx);
                    }
                }
            })
            .on_mouse_down(MouseButton::Right, {
                let deck = deck.clone();
                move |_event, _window, cx| {
                    if !deck.read(cx).is_editing() {
                        run(&deck, Deck::next, cx);
                    }
                }
            })
            .child(current_pane)
            .child(next_pane);

        let main_row = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .child(page_area)
            .children(notes_visible.then(|| self.notes_panel(cx, &p)));

        let projection_label = if projection {
            "Ocultar proyección"
        } else {
            "Mostrar proyección"
        };

        div()
            .flex()
            .flex_col()
            .gap(px(SPACING_MD))
            .p(px(WINDOW_PADDING))
            .size_full()
            .bg(p.bg)
            .text_size(px(FONT_SIZE_BODY))
            .text_color(p.fg)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.dispatch_key(event, window, cx);
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .flex_wrap()
                    .gap(px(SPACING_SM))
                    .px(px(SPACING_MD))
                    .py(px(SPACING_XXS))
                    .rounded(px(RADIUS_BUTTON))
                    .bg(p.headerbar_bg)
                    .child(
                        div()
                            .text_size(px(FONT_SIZE_BODY))
                            .text_color(p.muted)
                            .child(format!("Presenter - pagina {}/{total_label}", page + 1)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(SPACING_SM))
                            .child({
                                let open_disabled = self.dialog_open || compiling;
                                if open_disabled {
                                    control_disabled("btn-abrir", &p)
                                        .child("Abrir")
                                        .into_any_element()
                                } else {
                                    control_neutral("btn-abrir", &p)
                                        .child("Abrir")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _e, _w, cx| {
                                                this.open_file_dialog(cx)
                                            }),
                                        )
                                        .into_any_element()
                                }
                            })
                            .child(
                                control_neutral("btn-proyeccion", &p)
                                    .child(projection_label)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e, _w, cx| this.toggle_projection(cx)),
                                    ),
                            )
                            .child(if notes_visible {
                                control_filled("btn-notas", &p)
                                    .child("Notas")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e, _w, cx| {
                                            this.with_deck(cx, Deck::toggle_notes)
                                        }),
                                    )
                                    .into_any_element()
                            } else {
                                control_neutral("btn-notas", &p)
                                    .child("Notas")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e, _w, cx| {
                                            this.with_deck(cx, Deck::toggle_notes)
                                        }),
                                    )
                                    .into_any_element()
                            })
                            .child(action_button(
                                &deck,
                                "btn-tema",
                                "Tema",
                                &p,
                                Deck::toggle_theme,
                            )),
                    ),
            )
            .children(if compiling { None } else { error.clone() }.map(|e| {
                div()
                    .px(px(CONTROL_PADDING_X))
                    .py(px(SPACING_SM))
                    .rounded(px(RADIUS_BUTTON))
                    .bg(p.danger)
                    .text_color(p.on_danger)
                    .child(e)
            }))
            .child(
                div()
                    .flex()
                    .gap(px(SPACING_SM))
                    .child(nav_button(
                        &deck,
                        "nav-inicio",
                        "Inicio",
                        has_prev,
                        &p,
                        Deck::first,
                    ))
                    .child(nav_button(
                        &deck,
                        "nav-anterior",
                        "Anterior",
                        has_prev,
                        &p,
                        Deck::prev,
                    ))
                    .child(nav_button(
                        &deck,
                        "nav-siguiente",
                        "Siguiente",
                        has_next,
                        &p,
                        Deck::next,
                    ))
                    .child(nav_button(
                        &deck,
                        "nav-final",
                        "Final",
                        has_next,
                        &p,
                        Deck::last,
                    )),
            )
            .children(editing.then(|| {
                div()
                    .text_size(px(FONT_SIZE_CAPTION))
                    .text_color(p.accent_text)
                    .child("Editando nota — Esc para cancelar")
            }))
            .children(heading_editing.then(|| {
                div()
                    .text_size(px(FONT_SIZE_CAPTION))
                    .text_color(p.accent_text)
                    .child("Editando título — Enter para guardar, Esc para cancelar")
            }))
            .child(main_row)
    }
}

// ---------------------------------------------------------------------------
// Puente con la plataforma: `EntityInputHandler`.
//
// GPUI habla UTF-16 (`Range<usize>`) y el modelo `TextEdit` habla bytes, así
// que cada rango cruza la frontera con `TextEdit::{range_to_utf16,
// range_from_utf16, offset_to_utf16, offset_from_utf16, len_utf16}`. El editor
// destino lo marcan los flags de `Notes`: `editing` → cuerpo de la nota,
// `heading_editing` → título. Si el modelo (`note_edit`/`heading_edit`) aún no
// está creado (lo crea el punto de entrada a la edición, todavía WIP), la vista
// no acepta texto y los métodos no modifican nada.
//
// Tras cualquier cambio se vuelca el modelo a los buffers compartidos de
// `Notes` (`write_back`) y se avisa a la plataforma: `notify_text_input_changed`
// si cambió el contenido y `notify_text_selection_changed` si solo se movió el
// cursor o la selección, tal como pide el contrato del trait. `dirty` solo se
// marca cuando el texto cambió de verdad.
// ---------------------------------------------------------------------------

impl PresenterView {
    /// Reparte una pulsación: al editor si hay edición activa, a la navegación
    /// si no.
    ///
    /// En modo edición solo se consume el evento cuando el editor lo gestiona;
    /// un carácter imprimible se deja propagar para que la plataforma lo
    /// entregue al `EntityInputHandler` (`replace_text_in_range`). Insertarlo
    /// aquí lo duplicaría (ver `editor_command`).
    fn dispatch_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.active_target(cx).is_some() {
            if self.handle_edit_key(event, window, cx) {
                cx.stop_propagation();
            }
        } else {
            apply_nav(&self.deck, event, cx);
        }
    }

    /// Gestiona una pulsación en el editor activo. Devuelve `true` si la ha
    /// consumido (entonces no debe llegar a la navegación ni insertarse
    /// `key_char`).
    fn handle_edit_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(target) = self.active_target(cx) else {
            return false;
        };
        // El modelo se crea bajo demanda: si aún no ha corrido un frame tras
        // entrar en edición, la primera tecla ya encuentra editor.
        self.ensure_editor(target, cx);

        let keystroke = &event.keystroke;
        let command = editor_command(
            target,
            &keystroke.key,
            keystroke.key_char.as_deref(),
            keystroke.modifiers.shift,
            keystroke.modifiers.control,
        );

        match command {
            // Copiar/cortar la selección al portapapeles (3.12 (d)).
            EditorCommand::Copy => {
                self.copy_clipboard(target, false, window, cx);
                true
            }
            EditorCommand::Cut => {
                self.copy_clipboard(target, true, window, cx);
                true
            }
            // Texto imprimible: lo entrega la plataforma a `replace_text_in_range`.
            EditorCommand::PlatformText | EditorCommand::Ignore => false,
            EditorCommand::Cancel => {
                self.cancel_edit(target, window, cx);
                true
            }
            EditorCommand::Save => {
                self.save_heading(window, cx);
                true
            }
            EditorCommand::Paste => {
                self.paste_clipboard(target, window, cx);
                true
            }
            _ => {
                let mutates_text = match self.editor_mut(target) {
                    Some(edit) => {
                        apply_model_command(edit, command);
                        command_mutates_text(command)
                    }
                    None => false,
                };
                if mutates_text {
                    self.commit_text(target, window, cx);
                } else {
                    self.commit_selection(target, window, cx);
                }
                true
            }
        }
    }

    /// Cancela la edición activa y descarta el modelo (Esc).
    fn cancel_edit(&mut self, target: EditorTarget, window: &mut Window, cx: &mut Context<Self>) {
        self.with_deck(cx, |deck| match target {
            EditorTarget::Note => deck.notes.cancel_edit(),
            EditorTarget::Heading => deck.notes.cancel_edit_heading(),
        });
        self.drop_editor(target);
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    /// Confirma el título (Enter en el editor del título): vuelca el modelo a
    /// `Notes` y lo guarda.
    fn save_heading(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_editor(EditorTarget::Heading, cx);
        self.with_deck(cx, |deck| {
            if let Err(e) = deck.notes.save_heading() {
                deck.error = Some(e.to_string());
            }
        });
        self.drop_editor(EditorTarget::Heading);
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    /// Pega el portapapeles en la posición del cursor (Ctrl+V).
    ///
    /// Se conserva del editor anterior (3.9); copiar y cortar viven en
    /// `copy_clipboard` (3.12 (d)).
    fn paste_clipboard(
        &mut self,
        target: EditorTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let text = flatten_single_line(&text, target == EditorTarget::Heading);
        if let Some(edit) = self.editor_mut(target) {
            edit.replace_selection(&text);
        }
        self.commit_text(target, window, cx);
    }

    /// Copia (Ctrl+C) o corta (Ctrl+X) la selección al portapapeles.
    ///
    /// Sin selección es un no-op: no se escribe el portapapeles ni se marca la
    /// nota como sucia. Con `cut`, la selección se borra del modelo y el cambio
    /// se vuelca a `Notes`; copiar deja el documento intacto.
    fn copy_clipboard(
        &mut self,
        target: EditorTarget,
        cut: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = match self.editor(target) {
            Some(edit) => edit.selected_text().to_string(),
            None => return,
        };
        if text.is_empty() {
            return;
        }
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        if !cut {
            return;
        }
        if let Some(edit) = self.editor_mut(target) {
            edit.replace_selection("");
        }
        self.commit_text(target, window, cx);
    }

    /// Crea bajo demanda el modelo de edición del objetivo a partir del buffer
    /// de `Notes` (la entrada en edición y la primera tecla pueden llegar antes
    /// de que un frame haya creado el modelo).
    fn ensure_editor(&mut self, target: EditorTarget, cx: &mut Context<Self>) {
        if self.editor(target).is_some() {
            return;
        }
        let (text, cursor) = {
            let notes = &self.deck.read(cx).notes;
            match target {
                EditorTarget::Note => (notes.edit_buffer.clone(), notes.cursor),
                EditorTarget::Heading => (notes.heading_edit_buffer.clone(), notes.heading_cursor),
            }
        };
        let mut edit = TextEdit::new(text);
        edit.set_cursor(cursor);
        match target {
            EditorTarget::Note => self.note_edit = Some(edit),
            EditorTarget::Heading => self.heading_edit = Some(edit),
        }
    }

    /// Descarta el modelo y el layout conformado del objetivo.
    fn drop_editor(&mut self, target: EditorTarget) {
        match target {
            EditorTarget::Note => {
                self.note_edit = None;
                self.note_layout = None;
            }
            EditorTarget::Heading => {
                self.heading_edit = None;
                self.heading_layout = None;
            }
        }
    }

    /// Sincroniza el ciclo de vida de los modelos con los flags de `Notes`: los
    /// crea al entrar en edición y los descarta (con su layout) al salir.
    ///
    /// Al quedarse sin editores cierra también el arrastre en curso: salir del
    /// modo edición (Esc, guardar, cancelar) hace desaparecer el contenedor, el
    /// `mouse_up` del arrastre ya no llegará a ningún handler y el flag no puede
    /// quedarse pegado. La limpieza va aquí y no en `drop_editor` porque este se
    /// llama en cada frame para el objetivo que *no* se está editando.
    fn sync_editors(&mut self, cx: &mut Context<Self>) {
        let (editing, heading_editing) = {
            let notes = &self.deck.read(cx).notes;
            (notes.editing, notes.heading_editing)
        };
        if editing {
            self.ensure_editor(EditorTarget::Note, cx);
        } else {
            self.drop_editor(EditorTarget::Note);
        }
        if heading_editing {
            self.ensure_editor(EditorTarget::Heading, cx);
        } else {
            self.drop_editor(EditorTarget::Heading);
        }
        if !editing && !heading_editing {
            self.end_selection();
        }
    }

    /// Objetivo de edición activo según los flags de `Notes`.
    fn active_target(&self, cx: &App) -> Option<EditorTarget> {
        let notes = &self.deck.read(cx).notes;
        if notes.editing {
            Some(EditorTarget::Note)
        } else if notes.heading_editing {
            Some(EditorTarget::Heading)
        } else {
            None
        }
    }

    /// Modelo de texto del objetivo, si ya está creado.
    fn editor(&self, target: EditorTarget) -> Option<&TextEdit> {
        match target {
            EditorTarget::Note => self.note_edit.as_ref(),
            EditorTarget::Heading => self.heading_edit.as_ref(),
        }
    }

    /// Modelo de texto del objetivo, para mutarlo.
    fn editor_mut(&mut self, target: EditorTarget) -> Option<&mut TextEdit> {
        match target {
            EditorTarget::Note => self.note_edit.as_mut(),
            EditorTarget::Heading => self.heading_edit.as_mut(),
        }
    }

    /// Layout conformado en el último frame del objetivo.
    fn editor_layout(&self, target: EditorTarget) -> Option<&EditorLayout> {
        match target {
            EditorTarget::Note => self.note_layout.as_ref(),
            EditorTarget::Heading => self.heading_layout.as_ref(),
        }
    }

    /// `true` si el layout del objetivo se conformó con el texto que hay ahora
    /// en el modelo.
    ///
    /// Un layout de un frame anterior no sirve para calcular geometría: sus
    /// offsets ya no corresponderían al texto actual.
    fn layout_is_current(&self, target: EditorTarget) -> bool {
        match (self.editor_layout(target), self.editor(target)) {
            (Some(layout), Some(edit)) => layout.text.as_str() == edit.text.as_str(),
            _ => false,
        }
    }

    /// Vuelca el texto y el cursor del modelo activo en `Notes` (la fuente de
    /// verdad compartida con la app egui) sin copiar el texto.
    ///
    /// Devuelve `true` si el texto cambió de verdad; solo entonces `write_back`
    /// marca `dirty`.
    fn flush_editor(&mut self, target: EditorTarget, cx: &mut Context<Self>) -> bool {
        // Se desestructura la vista para poder prestar el editor y `deck` a la
        // vez (campos distintos) sin clonar el texto.
        let Self {
            deck,
            note_edit,
            heading_edit,
            ..
        } = self;
        let edit = match target {
            EditorTarget::Note => note_edit.as_ref(),
            EditorTarget::Heading => heading_edit.as_ref(),
        };
        let Some(edit) = edit else {
            return false;
        };
        deck.update(cx, |deck, _| {
            write_back(&mut deck.notes, target, &edit.text, edit.cursor())
        })
    }

    /// Confirma un cambio de **contenido**: vuelca el modelo, avisa a la
    /// plataforma con `notify_text_input_changed` y repinta la vista.
    fn commit_text(&mut self, target: EditorTarget, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_editor(target, cx);
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    /// Confirma un cambio que **no toca el texto** (solo cursor o selección):
    /// vuelca el modelo, avisa con `notify_text_selection_changed` y repinta.
    ///
    /// Nunca marca `dirty`: navegar por el texto no debe disparar el
    /// autoguardado.
    fn commit_selection(
        &mut self,
        target: EditorTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.flush_editor(target, cx);
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    /// Comienza una selección con el ratón: coloca el cursor en el byte que hay
    /// bajo `position` o, con `shift`, extiende la selección desde su ancla.
    ///
    /// Si el layout no corresponde al texto actual o el punto cae fuera de su
    /// rectángulo (`index_for_point` devuelve `None`) no hay byte que resolver y
    /// no se toca nada. Misma semántica que el `on_mouse_down` de la plantilla
    /// de GPUI.
    fn begin_selection(
        &mut self,
        target: EditorTarget,
        position: Point<Pixels>,
        shift: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.layout_is_current(target) {
            return;
        }
        let Some(byte) = self
            .editor_layout(target)
            .and_then(|layout| layout.index_for_point(position))
        else {
            return;
        };
        if let Some(edit) = self.editor_mut(target) {
            if shift {
                edit.select_to(byte);
            } else {
                edit.set_cursor(byte);
            }
        }
        self.is_selecting = true;
        self.focus_handle.focus(window, cx);
        self.commit_selection(target, window, cx);
    }

    /// Extiende la selección hasta el byte que hay bajo `position` durante un
    /// arrastre.
    ///
    /// Sin arrastre en curso no hace nada: un `mouse_move` sin botón pulsado no
    /// debe mover la selección. Igual que `begin_selection`, exige un layout
    /// vigente para no resolver bytes con offsets obsoletos.
    fn extend_selection(
        &mut self,
        target: EditorTarget,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_selecting || !self.layout_is_current(target) {
            return;
        }
        let Some(byte) = self
            .editor_layout(target)
            .and_then(|layout| layout.index_for_point(position))
        else {
            return;
        };
        if let Some(edit) = self.editor_mut(target) {
            edit.select_to(byte);
        }
        self.commit_selection(target, window, cx);
    }

    /// Cierra el arrastre de selección (botón izquierdo levantado, dentro o
    /// fuera del editor).
    fn end_selection(&mut self) {
        self.is_selecting = false;
    }
}

impl EntityInputHandler for PresenterView {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        let target = self.active_target(cx)?;
        let edit = self.editor(target)?;
        let bytes = edit.range_from_utf16(&range);
        // El rango ajustado se devuelve tal y como lo ha entendido el modelo
        // (acotado al texto y a fronteras de carácter).
        adjusted_range.replace(edit.range_to_utf16(&bytes));
        Some(clamped_slice(&edit.text, &bytes).to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let target = self.active_target(cx)?;
        let edit = self.editor(target)?;
        Some(UTF16Selection {
            range: selected_range_utf16(edit),
            reversed: edit.reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        let target = self.active_target(cx)?;
        let edit = self.editor(target)?;
        edit.marked
            .as_ref()
            .map(|marked| edit.range_to_utf16(marked))
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.active_target(cx) else {
            return;
        };
        let Some(edit) = self.editor_mut(target) else {
            return;
        };
        edit.unmark();
        self.commit_text(target, window, cx);
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.active_target(cx) else {
            return;
        };
        // El título es de una sola línea: los saltos se convierten en espacios.
        let single_line = target == EditorTarget::Heading;
        let Some(edit) = self.editor_mut(target) else {
            return;
        };
        replace_text_from_utf16(edit, range_utf16, text, single_line);
        self.commit_text(target, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.active_target(cx) else {
            return;
        };
        let new_text = flatten_single_line(new_text, target == EditorTarget::Heading);
        let Some(edit) = self.editor_mut(target) else {
            return;
        };
        let range = range_utf16
            .as_ref()
            .map(|range| edit.range_from_utf16(range));
        edit.replace_and_mark(range, &new_text, new_selected_range_utf16);
        self.commit_text(target, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let target = self.active_target(cx)?;
        if !self.layout_is_current(target) {
            return None;
        }
        let layout = self.editor_layout(target)?;
        let edit = self.editor(target)?;
        let bytes = edit.range_from_utf16(&range_utf16);
        // La geometría se ancla al rectángulo que la plataforma da en este
        // frame (`element_bounds`), que es el del elemento con foco.
        layout.bounds_for_byte_range(&bytes, element_bounds.origin)
    }

    fn selection_bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Bounds<Pixels>> {
        let Some(target) = self.active_target(cx) else {
            return Vec::new();
        };
        if !self.layout_is_current(target) {
            return Vec::new();
        }
        let (Some(layout), Some(edit)) = (self.editor_layout(target), self.editor(target)) else {
            return Vec::new();
        };
        let bytes = edit.range_from_utf16(&range_utf16);
        layout.selection_bounds_for_byte_range(&bytes, element_bounds.origin)
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        let target = self.active_target(cx)?;
        if !self.layout_is_current(target) {
            return None;
        }
        let layout = self.editor_layout(target)?;
        let edit = self.editor(target)?;
        let byte = layout.index_for_point(point)?;
        Some(edit.offset_to_utf16(byte))
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.active_target(cx) else {
            return;
        };
        let Some(edit) = self.editor_mut(target) else {
            return;
        };
        set_selection_from_utf16(edit, &range_utf16);
        self.commit_selection(target, window, cx);
    }

    fn text_length_utf16(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> Option<usize> {
        let target = self.active_target(cx)?;
        Some(self.editor(target)?.len_utf16())
    }

    fn accepts_text_input(&self, _window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.active_target(cx)
            .and_then(|target| self.editor(target))
            .is_some()
    }

    fn text_input_editable_range(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        let target = self.active_target(cx)?;
        Some(0..self.editor(target)?.len_utf16())
    }
}

// ---------------------------------------------------------------------------
// Tests del puente con la plataforma (`EntityInputHandler`).
//
// Usan el `TestPlatform` de GPUI (`gpui/test-support`, activada en
// `[dev-dependencies]`), que es headless: no pide display ni servidor gráfico.
// Cubren los escenarios del requisito «Edición de texto con selección e IME»
// de `openspec/changes/migracion-gpui/specs/gpui-ui/spec.md`.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod input_handler_tests {
    use super::*;
    use gpui::{Keystroke, Modifiers, TestAppContext, VisualTestContext};

    /// Crea un `PresenterView` con el editor `target` activo y un `TextEdit` con
    /// `text`, sobre un deck vacío y sin proyección (así el test no abre una
    /// segunda ventana de audiencia).
    fn presenter_editing<'a>(
        cx: &'a mut TestAppContext,
        target: EditorTarget,
        text: &str,
    ) -> (Entity<PresenterView>, &'a mut VisualTestContext) {
        let mut deck = Deck::empty();
        deck.projection = false;
        match target {
            EditorTarget::Note => {
                deck.notes.editing = true;
                deck.notes.edit_buffer = text.to_string();
                deck.notes.cursor = text.len();
            }
            EditorTarget::Heading => {
                deck.notes.heading_editing = true;
                deck.notes.heading_edit_buffer = text.to_string();
                deck.notes.heading_cursor = text.len();
            }
        }

        let deck = cx.update(|app| app.new(|_| deck));
        let (view, cx) = cx.add_window_view(|window, cx| PresenterView::new(deck, window, cx));
        view.update_in(cx, |view, _window, _cx| match target {
            EditorTarget::Note => view.note_edit = Some(TextEdit::new(text)),
            EditorTarget::Heading => view.heading_edit = Some(TextEdit::new(text)),
        });
        (view, cx)
    }

    /// Conforma `text` en una línea con el `PlatformTextSystem` del test y la
    /// envuelve en el `EditorLayout` que el `TextElement` escribe en cada
    /// frame. Los avances del text system de test son ficticios pero
    /// monótonos, suficiente para comprobar el mapeo punto ↔ byte.
    fn layout_for(text: &str, window: &mut Window) -> EditorLayout {
        let run = TextRun {
            len: text.len(),
            font: gpui::font("Helvetica"),
            color: gpui::hsla(0., 0., 0., 1.),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line =
            window
                .text_system()
                .shape_line(text.into(), px(14.), std::slice::from_ref(&run), None);
        EditorLayout {
            lines: vec![(0..text.len(), line)],
            bounds: Bounds::new(point(px(0.), px(0.)), gpui::size(px(400.), px(40.))),
            line_height: px(20.),
            text: text.into(),
        }
    }

    #[gpui::test]
    fn replace_text_in_range_updates_model_and_notes_buffers(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        view.update_in(cx, |view, window, cx| {
            // "mundo" son los bytes 5..10 y las unidades UTF-16 5..10.
            view.replace_text_in_range(Some(5..10), "Rust", window, cx);

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.text, "hola Rust");
            assert_eq!(edit.cursor(), 9);
            assert_eq!(edit.selection, 9..9);

            let notes = &view.deck.read(cx).notes;
            assert_eq!(notes.edit_buffer, "hola Rust");
            assert_eq!(notes.cursor, 9);
            assert!(notes.dirty, "el cuerpo de la nota marca dirty al escribir");
        });
    }

    #[gpui::test]
    fn replace_text_in_range_without_range_uses_the_selection(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        view.update_in(cx, |view, window, cx| {
            let edit = view.note_edit.as_mut().expect("editor de la nota");
            edit.set_cursor(5);
            edit.select_to(10);

            view.replace_text_in_range(None, "Rust", window, cx);

            assert_eq!(
                view.note_edit.as_ref().expect("editor de la nota").text,
                "hola Rust"
            );
            assert_eq!(view.deck.read(cx).notes.edit_buffer, "hola Rust");
        });
    }

    #[gpui::test]
    fn select_all_then_replace_replaces_whole_content(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "nota vieja");
        view.update_in(cx, |view, window, cx| {
            view.note_edit
                .as_mut()
                .expect("editor de la nota")
                .select_all();
            let selection = view
                .selected_text_range(false, window, cx)
                .expect("hay editor activo");
            // Selección completa: 10 bytes = 10 unidades UTF-16.
            assert_eq!(selection.range, 0..10);
            assert!(!selection.reversed);

            view.replace_text_in_range(None, "nueva", window, cx);

            assert_eq!(
                view.note_edit.as_ref().expect("editor de la nota").text,
                "nueva"
            );
            let notes = &view.deck.read(cx).notes;
            assert_eq!(notes.edit_buffer, "nueva");
            assert_eq!(notes.cursor, 5);
        });
    }

    #[gpui::test]
    fn replace_text_in_range_with_full_range_replaces_whole_content(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "otra nota larga");
        view.update_in(cx, |view, window, cx| {
            // El rango completo tal y como lo envía la plataforma tras Ctrl+A.
            view.replace_text_in_range(Some(0..15), "final", window, cx);

            assert_eq!(
                view.note_edit.as_ref().expect("editor de la nota").text,
                "final"
            );
            assert_eq!(view.deck.read(cx).notes.edit_buffer, "final");
        });
    }

    #[gpui::test]
    fn selected_text_range_is_utf16_for_accents(cx: &mut TestAppContext) {
        // "café": 5 bytes, 4 unidades UTF-16.
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "café");
        view.update_in(cx, |view, window, cx| {
            assert_eq!(view.text_length_utf16(window, cx), Some(4));

            view.note_edit
                .as_mut()
                .expect("editor de la nota")
                .select_all();
            let all = view
                .selected_text_range(false, window, cx)
                .expect("hay editor activo");
            assert_eq!(all.range, 0..4);

            // "é" son los bytes 3..5 y la unidad UTF-16 3..4.
            view.set_selected_text_range(3..4, window, cx);
            let accent = view
                .selected_text_range(false, window, cx)
                .expect("hay editor activo");
            assert_eq!(accent.range, 3..4);

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.selection, 3..5);
            assert_eq!(edit.selected_text(), "é");
        });
    }

    #[gpui::test]
    fn selected_text_range_is_utf16_outside_bmp(cx: &mut TestAppContext) {
        // "A🙂Z": 6 bytes, 4 unidades UTF-16 (el emoji es un par subrogado).
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            assert_eq!(view.text_length_utf16(window, cx), Some(4));

            view.set_selected_text_range(1..3, window, cx);
            let emoji = view
                .selected_text_range(false, window, cx)
                .expect("hay editor activo");
            assert_eq!(emoji.range, 1..3);

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.selection, 1..5);
            assert_eq!(edit.selected_text(), "🙂");
        });
    }

    #[gpui::test]
    fn set_selected_text_range_clamps_to_the_text(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            // Un rango que se pasa del final se acota a la longitud del texto.
            view.set_selected_text_range(3..99, window, cx);

            let clamped = view
                .selected_text_range(false, window, cx)
                .expect("hay editor activo");
            assert_eq!(clamped.range, 3..4);
            assert_eq!(
                view.note_edit
                    .as_ref()
                    .expect("editor de la nota")
                    .selection,
                5..6
            );
        });
    }

    /// Cambiar la selección (o desmarcar una composición) no toca el texto: la
    /// nota no debe quedar `dirty`, o navegar dispararía el autoguardado.
    #[gpui::test]
    fn set_selected_text_range_does_not_mark_dirty(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        view.update_in(cx, |view, window, cx| {
            view.set_selected_text_range(0..4, window, cx);

            let notes = &view.deck.read(cx).notes;
            assert!(!notes.dirty, "cambiar la selección no marca dirty");
            assert_eq!(notes.edit_buffer, "hola mundo");
            // El cursor sí se propaga (extremo final de la selección, bytes).
            assert_eq!(notes.cursor, 4);

            // Desmarcar una composición IME tampoco ensucia si el texto no cambió.
            view.unmark_text(window, cx);
            assert!(!view.deck.read(cx).notes.dirty, "desmarcar no marca dirty");
        });
    }

    #[gpui::test]
    fn text_for_range_returns_the_slice_and_adjusted_range(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            // El emoji ocupa 4 bytes y 2 unidades UTF-16.
            let mut adjusted = None;
            let emoji = view.text_for_range(1..3, &mut adjusted, window, cx);
            assert_eq!(emoji.as_deref(), Some("🙂"));
            assert_eq!(adjusted, Some(1..3), "el rango ajustado no cambia");

            // Un rango que se pasa del final se acota al texto: el rango
            // ajustado refleja lo que el modelo ha entendido de verdad.
            let mut adjusted = None;
            let clamped = view.text_for_range(2..99, &mut adjusted, window, cx);
            assert_eq!(clamped.as_deref(), Some("Z"));
            assert_eq!(adjusted, Some(3..4));

            // Sin editor activo no hay texto ni rango ajustado.
            view.deck.update(cx, |deck, _| deck.notes.editing = false);
            let mut adjusted = None;
            assert_eq!(view.text_for_range(0..1, &mut adjusted, window, cx), None);
            assert_eq!(adjusted, None);
        });
    }

    #[gpui::test]
    fn bounds_for_range_maps_utf16_to_geometry(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            let element_bounds = Bounds::new(point(px(10.), px(5.)), gpui::size(px(400.), px(40.)));

            // Sin layout, la geometría no tiene respuesta. (La ventana ya ha
            // pintado el `TextElement`, así que el layout se descarta a mano.)
            view.note_layout = None;
            assert_eq!(
                view.bounds_for_range(1..3, element_bounds, window, cx),
                None
            );

            view.note_layout = Some(layout_for("A🙂Z", window));
            let (x1, x5, x6, line_height) = {
                let layout = view.note_layout.as_ref().expect("layout");
                let line = &layout.lines[0].1;
                (
                    line.x_for_index(1),
                    line.x_for_index(5),
                    line.x_for_index(6),
                    layout.line_height,
                )
            };

            // El emoji: unidades UTF-16 1..3 → bytes 1..5.
            let bounds = view
                .bounds_for_range(1..3, element_bounds, window, cx)
                .expect("hay layout del editor con el texto actual");
            assert_eq!(
                bounds,
                Bounds::from_corners(
                    point(px(10.) + x1, px(5.)),
                    point(px(10.) + x5, px(5.) + line_height),
                )
            );

            // Un rango fuera del texto se acota a la longitud del editor.
            let clamped = view
                .bounds_for_range(2..99, element_bounds, window, cx)
                .expect("hay layout del editor");
            assert_eq!(
                clamped,
                Bounds::from_corners(
                    point(px(10.) + x5, px(5.)),
                    point(px(10.) + x6, px(5.) + line_height),
                )
            );

            // Un layout de un texto anterior se descarta.
            view.note_layout = Some(layout_for("otro texto", window));
            assert_eq!(
                view.bounds_for_range(1..3, element_bounds, window, cx),
                None
            );
        });
    }

    #[gpui::test]
    fn selection_bounds_for_range_maps_utf16_to_geometry(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            let element_bounds = Bounds::new(point(px(4.), px(2.)), gpui::size(px(400.), px(40.)));

            // Sin layout no hay rectángulos de selección. (La ventana ya ha
            // pintado el `TextElement`; se descarta el layout a mano.)
            view.note_layout = None;
            assert!(
                view.selection_bounds_for_range(0..4, element_bounds, window, cx)
                    .is_empty()
            );

            view.note_layout = Some(layout_for("A🙂Z", window));

            // Selección completa (4 unidades UTF-16 → 6 bytes): una línea, un
            // rectángulo que la cubre entera.
            let rects = view.selection_bounds_for_range(0..4, element_bounds, window, cx);
            assert_eq!(rects.len(), 1, "el texto ocupa una sola línea");
            let (start_x, end_x, line_height) = {
                let layout = view.note_layout.as_ref().expect("layout");
                let line = &layout.lines[0].1;
                (line.x_for_index(0), line.x_for_index(6), layout.line_height)
            };
            assert_eq!(
                rects[0],
                Bounds::from_corners(
                    point(px(4.) + start_x, px(2.)),
                    point(px(4.) + end_x, px(2.) + line_height),
                )
            );

            // Un layout obsoleto tampoco sirve.
            view.note_layout = Some(layout_for("otro texto", window));
            assert!(
                view.selection_bounds_for_range(0..4, element_bounds, window, cx)
                    .is_empty()
            );
        });
    }

    /// Regresión: `paste` (default del trait) delega en
    /// `replace_text_in_range(None, ..)`, que debe sustituir la selección activa
    /// y no solo insertar al final del texto.
    #[gpui::test]
    fn paste_with_an_active_selection_replaces_it(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            // Selecciona el emoji (bytes 1..5 = unidades UTF-16 1..3).
            view.set_selected_text_range(1..3, window, cx);
            assert!(
                !view.deck.read(cx).notes.dirty,
                "seleccionar no ensucia la nota"
            );

            view.paste(
                gpui::ClipboardItem::new_string("ok".to_string()),
                window,
                cx,
            );

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.text, "AokZ", "el pegado sustituye la selección");
            assert_eq!(edit.selection, 3..3);

            let notes = &view.deck.read(cx).notes;
            assert_eq!(notes.edit_buffer, "AokZ");
            assert_eq!(notes.cursor, 3);
            assert!(notes.dirty, "pegar sí cambia el texto");
        });
    }

    #[gpui::test]
    fn character_index_for_point_roundtrips_bytes_and_utf16(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            view.note_layout = Some(layout_for("A🙂Z", window));
            assert!(
                view.layout_is_current(EditorTarget::Note),
                "el layout se conformó con el texto actual"
            );

            let line_height = view.note_layout.as_ref().expect("layout").line_height;
            // Fronteras de carácter del texto, con la x que les da el layout.
            let boundaries: Vec<(usize, Pixels)> = {
                let line = &view.note_layout.as_ref().expect("layout").lines[0].1;
                let mut boundaries = Vec::new();
                let mut byte = 0;
                for ch in "A🙂Z".chars() {
                    boundaries.push((byte, line.x_for_index(byte)));
                    byte += ch.len_utf8();
                }
                boundaries.push((byte, line.x_for_index(byte)));
                boundaries
            };
            assert_eq!(
                boundaries.iter().map(|(byte, _)| *byte).collect::<Vec<_>>(),
                vec![0, 1, 5, 6],
                "A=1 byte, 🙂=4 bytes, Z=1 byte"
            );

            for (byte, x) in boundaries {
                let utf16 = view
                    .character_index_for_point(point(x, line_height / 2.), window, cx)
                    .expect("hay layout y editor activo");
                let expected = "A🙂Z"[..byte].encode_utf16().count();
                assert_eq!(
                    utf16, expected,
                    "el byte {byte} es la unidad UTF-16 {utf16}"
                );
            }

            // Justo después del final del texto, pero dentro del editor: el
            // índice se acota a la longitud (4 unidades UTF-16).
            let text_end_x = view
                .note_layout
                .as_ref()
                .expect("layout")
                .lines
                .first()
                .expect("una línea")
                .1
                .width;
            let past_end = view
                .character_index_for_point(
                    point(text_end_x + px(10.), line_height / 2.),
                    window,
                    cx,
                )
                .expect("hay layout y editor activo");
            assert_eq!(past_end, 4);

            // Fuera del rectángulo del editor no hay posición: `None`.
            let outside = view.character_index_for_point(point(px(-50.), px(-50.)), window, cx);
            assert_eq!(outside, None);
        });
    }

    #[gpui::test]
    fn character_index_for_point_needs_a_current_layout(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            // Sin layout todavía, la geometría no tiene respuesta.
            assert_eq!(
                view.character_index_for_point(point(px(0.), px(0.)), window, cx),
                None
            );

            // Un layout de un texto anterior se descarta.
            view.note_layout = Some(layout_for("otro texto", window));
            assert!(!view.layout_is_current(EditorTarget::Note));
            assert_eq!(
                view.character_index_for_point(point(px(0.), px(0.)), window, cx),
                None
            );
        });
    }

    #[gpui::test]
    fn heading_editor_is_single_line(cx: &mut TestAppContext) {
        // "Título": 7 bytes, 6 unidades UTF-16.
        let (view, cx) = presenter_editing(cx, EditorTarget::Heading, "Título");
        view.update_in(cx, |view, window, cx| {
            view.replace_text_in_range(Some(0..6), "linea\nnueva", window, cx);

            assert_eq!(
                view.heading_edit.as_ref().expect("editor del título").text,
                "linea nueva",
                "en el título los saltos de línea se aplanan a espacios"
            );
            let notes = &view.deck.read(cx).notes;
            assert_eq!(notes.heading_edit_buffer, "linea nueva");
            assert!(!notes.dirty, "el título no marca dirty: es del cuerpo");
        });
    }

    #[gpui::test]
    fn ime_marked_range_then_unmark(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "A🙂Z");
        view.update_in(cx, |view, window, cx| {
            view.replace_and_mark_text_in_range(Some(1..3), "é🙂", Some(1..3), window, cx);

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.text, "Aé🙂Z");
            assert_eq!(edit.marked, Some(1..7));
            assert_eq!(edit.selection, 3..7);
            // Bytes 1..7 → unidades UTF-16 1..4.
            assert_eq!(view.marked_text_range(window, cx), Some(1..4));
            assert_eq!(view.deck.read(cx).notes.edit_buffer, "Aé🙂Z");

            // Confirmación de la composición: se descarta el rango marcado.
            view.unmark_text(window, cx);
            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.marked, None);
            assert_eq!(edit.text, "Aé🙂Z");
            assert_eq!(view.marked_text_range(window, cx), None);
        });
    }

    #[gpui::test]
    fn editing_flags_gate_the_input_handler(cx: &mut TestAppContext) {
        let mut deck = Deck::empty();
        deck.projection = false;
        let deck = cx.update(|app| app.new(|_| deck));
        let (view, cx) = cx.add_window_view(|window, cx| PresenterView::new(deck, window, cx));

        view.update_in(cx, |view, window, cx| {
            // Sin flag de edición la vista no acepta texto ni escribe nada.
            assert!(!view.accepts_text_input(window, cx));
            assert_eq!(view.active_target(cx), None);
            assert_eq!(view.text_length_utf16(window, cx), None);
            assert_eq!(view.text_input_editable_range(window, cx), None);

            view.replace_text_in_range(Some(0..0), "no debería entrar", window, cx);
            assert_eq!(view.note_edit, None);
            assert_eq!(view.deck.read(cx).notes.edit_buffer, "");
        });

        // Con `editing` el objetivo es el cuerpo, pero sin modelo de texto
        // creado (`note_edit`) la vista todavía no acepta entrada.
        view.update(cx, |view, cx| {
            view.deck.update(cx, |deck, _| deck.notes.editing = true);
        });
        view.update_in(cx, |view, window, cx| {
            assert_eq!(view.active_target(cx), Some(EditorTarget::Note));
            assert!(
                !view.accepts_text_input(window, cx),
                "sin modelo de texto no se acepta entrada"
            );

            view.note_edit = Some(TextEdit::new("cuerpo"));
            assert!(view.accepts_text_input(window, cx));

            view.deck.update(cx, |deck, _| {
                deck.notes.editing = false;
                deck.notes.heading_editing = true;
            });
            assert_eq!(view.active_target(cx), Some(EditorTarget::Heading));
            assert!(
                !view.accepts_text_input(window, cx),
                "el título tampoco acepta texto hasta que exista su modelo"
            );

            view.heading_edit = Some(TextEdit::new("Título"));
            assert!(
                view.accepts_text_input(window, cx),
                "el título también acepta texto"
            );
        });
    }

    /// En Wayland/X11 el backend puede entregar `key_char` junto con Esc o
    /// Enter; esos comandos deben gestionarse igual y **no** delegarse como
    /// texto, o el carácter acabaría insertado dos veces.
    #[test]
    fn escape_and_enter_never_insert_key_char() {
        for key_char in [None, Some(""), Some("\u{1b}"), Some("\r")] {
            assert_eq!(
                editor_command(EditorTarget::Note, "escape", key_char, false, false),
                EditorCommand::Cancel,
            );
        }
        assert_eq!(
            editor_command(EditorTarget::Note, "enter", Some("\r"), false, false),
            EditorCommand::Newline,
        );
        assert_eq!(
            editor_command(EditorTarget::Heading, "enter", Some("\r"), false, false),
            EditorCommand::Save,
        );

        // Y ninguno de esos comandos toca el modelo de texto desde el handler de
        // teclas: el `key_char` llegará por `replace_text_in_range`.
        let mut edit = TextEdit::new("nota");
        for command in [EditorCommand::Cancel, EditorCommand::Save] {
            assert!(!command_mutates_text(command));
            assert!(!apply_model_command(&mut edit, command));
        }
        assert_eq!(edit.text, "nota");
    }

    /// Un carácter imprimible se delega a la plataforma (`PlatformText`), que lo
    /// entrega como `replace_text_in_range(None, key_char)`.
    #[test]
    fn printable_key_is_delegated_to_the_input_handler() {
        let command = editor_command(EditorTarget::Note, "a", Some("a"), false, false);
        assert_eq!(command, EditorCommand::PlatformText);
        assert!(!command_mutates_text(command));

        let mut edit = TextEdit::new("hola");
        assert!(!apply_model_command(&mut edit, command));
        assert_eq!(
            edit.text, "hola",
            "el handler de teclas no inserta el texto"
        );

        // Los atajos de control sí se resuelven en el modelo.
        assert_eq!(
            editor_command(EditorTarget::Note, "a", Some("a"), false, true),
            EditorCommand::SelectAll,
        );
        assert!(apply_model_command(&mut edit, EditorCommand::SelectAll));
        assert_eq!(edit.selection, 0..4);

        // Una tecla sin carácter y sin comando no hace nada.
        assert_eq!(
            editor_command(EditorTarget::Note, "f5", None, false, false),
            EditorCommand::Ignore,
        );
    }

    // -----------------------------------------------------------------------
    // (e) Reencaminado del teclado y creación de los modelos de edición.
    // -----------------------------------------------------------------------

    /// Evento tal y como lo entrega el backend para una tecla, `key_char` incluido.
    fn key_down(key: &str, key_char: Option<&str>, modifiers: Modifiers) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke {
                modifiers,
                key: key.to_string(),
                key_char: key_char.map(str::to_string),
            },
            is_held: false,
            prefer_character_input: false,
        }
    }

    #[gpui::test]
    fn escape_cancels_editing_without_inserting_text(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola");
        view.update_in(cx, |view, window, cx| {
            let event = key_down("escape", Some("\u{1b}"), Modifiers::default());
            assert!(
                view.handle_edit_key(&event, window, cx),
                "el editor consume Esc"
            );

            let notes = &view.deck.read(cx).notes;
            assert!(!notes.editing, "Esc sale del modo edición");
            assert_eq!(notes.edit_buffer, "", "cancelar limpia el buffer");
            assert_eq!(view.note_edit, None, "el modelo se descarta al cancelar");
            assert!(view.note_layout.is_none(), "y su layout conformado también");
            assert!(!notes.dirty, "cancelar no marca la nota como sucia");
        });
    }

    #[gpui::test]
    fn enter_inserts_exactly_one_newline_in_the_note(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "linea");
        view.update_in(cx, |view, window, cx| {
            // El backend entrega `key_char = "\n"` junto con Enter: si el handler
            // de teclas lo insertara aquí, habría dos saltos de línea.
            let event = key_down("enter", Some("\n"), Modifiers::default());
            assert!(view.handle_edit_key(&event, window, cx));

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.text, "linea\n", "un solo salto de línea");
            assert_eq!(edit.cursor(), 6);

            let notes = &view.deck.read(cx).notes;
            assert_eq!(notes.edit_buffer, "linea\n");
            assert!(notes.dirty, "insertar texto marca la nota como sucia");
        });
    }

    #[gpui::test]
    fn enter_in_the_heading_saves_without_inserting_it(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Heading, "Titulo");
        view.update_in(cx, |view, window, cx| {
            // El backend manda `\r` como `key_char` de Enter.
            let event = key_down("enter", Some("\r"), Modifiers::default());
            assert!(view.handle_edit_key(&event, window, cx));

            let notes = &view.deck.read(cx).notes;
            assert!(!notes.heading_editing, "Enter en el título guarda y sale");
            assert_eq!(notes.heading.as_deref(), Some("Titulo"));
            assert!(!notes.heading_edit_buffer.contains('\r'));
            assert_eq!(view.heading_edit, None);
        });
    }

    /// El invariante anti-doble-inserción: el handler de teclas **no** consume ni
    /// inserta un carácter imprimible; lo hace la plataforma con `key_char`.
    #[gpui::test]
    fn printable_key_is_inserted_once_by_the_platform(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "");
        view.update_in(cx, |view, window, cx| {
            let event = key_down("a", Some("a"), Modifiers::default());
            assert!(
                !view.handle_edit_key(&event, window, cx),
                "el evento se deja propagar para que la plataforma entregue key_char"
            );
            assert_eq!(
                view.note_edit.as_ref().expect("editor").text,
                "",
                "el handler de teclas no inserta nada"
            );

            // Lo que hace la plataforma después: entregar el carácter con foco.
            view.replace_text_in_range(None, "a", window, cx);

            assert_eq!(
                view.note_edit.as_ref().expect("editor").text,
                "a",
                "exactamente una inserción"
            );
            assert_eq!(view.deck.read(cx).notes.edit_buffer, "a");
        });
    }

    #[gpui::test]
    fn shift_arrow_extends_the_selection_in_the_model(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "abc");
        view.update_in(cx, |view, window, cx| {
            let event = key_down(
                "left",
                None,
                Modifiers {
                    shift: true,
                    ..Default::default()
                },
            );
            assert!(view.handle_edit_key(&event, window, cx));

            let edit = view.note_edit.as_ref().expect("editor");
            assert_eq!(edit.selection, 2..3, "Shift+← extiende un carácter");
            assert!(edit.reversed, "el ancla se mantiene en el extremo opuesto");
            assert!(
                !view.deck.read(cx).notes.dirty,
                "seleccionar no marca la nota como sucia"
            );
        });
    }

    #[gpui::test]
    fn arrows_sync_the_caret_with_notes_without_marking_dirty(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "abc");
        view.update_in(cx, |view, window, cx| {
            let event = key_down("left", None, Modifiers::default());
            assert!(view.handle_edit_key(&event, window, cx));

            assert_eq!(view.note_edit.as_ref().expect("editor").cursor(), 2);
            let notes = &view.deck.read(cx).notes;
            assert_eq!(notes.cursor, 2, "el cursor se sincroniza con Notes");
            assert!(!notes.dirty);
        });
    }

    #[gpui::test]
    fn handle_edit_key_creates_the_model_on_demand(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "nota");
        // Simula la primera tecla antes de que un frame haya creado el editor.
        view.update_in(cx, |view, _window, _cx| view.note_edit = None);

        view.update_in(cx, |view, window, cx| {
            let event = key_down("home", None, Modifiers::default());
            assert!(view.handle_edit_key(&event, window, cx));

            let edit = view
                .note_edit
                .as_ref()
                .expect("el modelo se crea bajo demanda");
            assert_eq!(edit.text, "nota", "copiado del buffer de Notes");
            assert_eq!(edit.cursor(), 0, "el comando se aplica al modelo nuevo");
        });
    }

    #[gpui::test]
    fn handle_edit_key_ignores_keys_without_an_active_editor(cx: &mut TestAppContext) {
        let mut deck = Deck::empty();
        deck.projection = false;
        let deck = cx.update(|app| app.new(|_| deck));
        let (view, cx) = cx.add_window_view(|window, cx| PresenterView::new(deck, window, cx));

        view.update_in(cx, |view, window, cx| {
            let event = key_down("left", None, Modifiers::default());
            assert!(
                !view.handle_edit_key(&event, window, cx),
                "sin edición el editor no consume nada"
            );
            assert_eq!(view.note_edit, None);
        });
    }

    /// Escribir en el editor no debe disparar los atajos de navegación (h/l/j/k),
    /// que es el motivo por el que el teclado se gestiona con `on_key_down`.
    #[gpui::test]
    fn typing_does_not_navigate_while_editing(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "nota");
        view.update(cx, |view, cx| {
            view.deck.update(cx, |deck, _| deck.total = 3);
        });

        view.update_in(cx, |view, window, cx| {
            let event = key_down("l", Some("l"), Modifiers::default());
            view.dispatch_key(&event, window, cx);
            assert_eq!(view.deck.read(cx).page, 0, "«l» no pasa de página");
            assert_eq!(
                view.note_edit.as_ref().expect("editor").text,
                "nota",
                "y tampoco se inserta en el handler de teclas"
            );
        });

        // Fuera de edición la misma tecla sí navega.
        view.update_in(cx, |view, window, cx| {
            view.deck.update(cx, |deck, _| deck.notes.editing = false);
            let event = key_down("l", Some("l"), Modifiers::default());
            view.dispatch_key(&event, window, cx);
            assert_eq!(view.deck.read(cx).page, 1);
        });
    }

    /// Camino completo con el `TestPlatform`: la ventana pinta el `TextElement`,
    /// este registra el `ElementInputHandler` y la plataforma entrega `key_char`
    /// (igual que el backend real). Cada carácter debe llegar una sola vez y
    /// Enter, que sí consume el editor, ninguna.
    #[gpui::test]
    fn typing_over_the_window_inserts_each_character_once(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "");
        cx.run_until_parked();
        cx.simulate_keystrokes("a space b enter");

        view.update_in(cx, |view, _window, cx| {
            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.text, "a b\n", "ni caracteres de más ni de menos");
            assert_eq!(view.deck.read(cx).notes.edit_buffer, "a b\n");
        });
    }

    /// `TextElement::paint` guarda el layout conformado en la vista: es la
    /// entrada de la geometría del IME y del hit-testing del ratón.
    #[gpui::test]
    fn text_element_saves_the_layout_in_the_view(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "ab\ncde");
        cx.run_until_parked();

        view.update_in(cx, |view, _window, _cx| {
            let layout = view
                .note_layout
                .as_ref()
                .expect("el TextElement guarda su layout al pintar");
            assert_eq!(layout.lines.len(), 2, "una línea por cada «\\n»");
            assert_eq!(layout.text.as_str(), "ab\ncde");
            assert!(
                view.layout_is_current(EditorTarget::Note),
                "el layout guardado corresponde al texto del modelo"
            );
        });
    }

    /// El texto de cada objetivo vive en su propio layout: el del título no pisa
    /// el de la nota ni al revés.
    #[gpui::test]
    fn heading_element_saves_its_own_layout(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Heading, "Titulo");
        cx.run_until_parked();

        view.update_in(cx, |view, _window, _cx| {
            let layout = view
                .heading_layout
                .as_ref()
                .expect("el TextElement del título guarda su layout");
            assert_eq!(layout.lines.len(), 1, "el título es de una sola línea");
            assert_eq!(layout.text.as_str(), "Titulo");
            assert!(view.note_layout.is_none(), "la nota no se ha editado");
        });
    }

    // -----------------------------------------------------------------------
    // Cálculo del frame del `TextElement` (conformado, resaltado y cursor).
    // -----------------------------------------------------------------------

    #[gpui::test]
    fn editor_frame_splits_lines_and_maps_points(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "ab\ncde");
        view.update_in(cx, |view, window, _cx| {
            let edit = view.note_edit.as_ref().expect("editor");
            let bounds = Bounds::new(point(px(0.), px(0.)), size(px(200.), px(40.)));
            let frame = editor_frame(
                &edit.text,
                &edit.selection.clone(),
                edit.marked.as_ref(),
                edit.cursor(),
                bounds,
                window,
                &palette(false),
            );

            let layout = frame.layout.expect("layout conformado");
            assert_eq!(layout.lines.len(), 2);
            assert_eq!(layout.text.as_str(), "ab\ncde");
            // Mitad inferior del rectángulo → segunda línea → byte 3.
            let line_height = layout.line_height;
            assert_eq!(
                layout.index_for_point(point(px(0.), line_height * 1.5)),
                Some(3)
            );
            assert_eq!(layout.index_for_point(point(px(0.), px(0.))), Some(0));

            assert!(frame.selection.is_empty(), "no hay selección");
            assert!(frame.marked.is_empty(), "no hay pre-edit de IME");
            let caret = frame.caret.expect("con la selección colapsada hay cursor");
            assert_eq!(
                caret.bounds.origin.y, line_height,
                "el cursor está en la segunda línea"
            );
            assert_eq!(caret.bounds.size.width, px(2.));
        });
    }

    #[gpui::test]
    fn editor_frame_highlights_a_multiline_selection(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "ab\ncde");
        view.update_in(cx, |view, window, _cx| {
            let edit = view.note_edit.as_ref().expect("editor");
            let bounds = Bounds::new(point(px(0.), px(0.)), size(px(200.), px(40.)));
            let selection = 0..5; // "ab\ncd": toca las dos líneas
            let frame = editor_frame(
                &edit.text,
                &selection,
                None,
                selection.end,
                bounds,
                window,
                &palette(false),
            );

            assert_eq!(
                frame.selection.len(),
                2,
                "un rectángulo de selección por línea"
            );
            assert!(frame.caret.is_none(), "con selección no se pinta el cursor");
        });
    }

    #[gpui::test]
    fn editor_frame_underlines_the_ime_marked_range(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "ab\ncde");
        view.update_in(cx, |view, window, _cx| {
            let edit = view.note_edit.as_ref().expect("editor");
            let bounds = Bounds::new(point(px(0.), px(0.)), size(px(200.), px(40.)));
            let marked = 3..5;
            let frame = editor_frame(
                &edit.text,
                &(5..5),
                Some(&marked),
                marked.end,
                bounds,
                window,
                &palette(false),
            );

            assert_eq!(frame.marked.len(), 1, "un subrayado por el pre-edit");
            let underline = frame.marked[0].bounds;
            assert_eq!(underline.size.height, px(1.));
            let line_height = frame.layout.as_ref().expect("layout").line_height;
            assert_eq!(
                underline.bottom(),
                bounds.origin.y + line_height * 2.,
                "el subrayado va al pie de la línea del rango marcado"
            );
        });
    }

    // -----------------------------------------------------------------------
    // Selección con ratón (clic, arrastre y Shift+clic).
    //
    // Ejercitan el cableado end-to-end: `simulate_mouse_*` despacha eventos de
    // plataforma que el hit-testing de GPUI entrega al contenedor del editor,
    // cuyos handlers llaman a `begin_selection`/`extend_selection`. El punto se
    // deriva del layout conformado (coordenadas de ventana), así que el byte
    // golpeado es el esperado.
    // -----------------------------------------------------------------------

    /// Punto (coordenadas de ventana) dentro del editor `target` que cae sobre el
    /// byte `byte`, según el layout conformado del último frame.
    fn point_for_byte(view: &PresenterView, target: EditorTarget, byte: usize) -> Point<Pixels> {
        let layout = view
            .editor_layout(target)
            .expect("el editor tiene layout tras el primer frame");
        let index = layout.line_index_for_byte(byte);
        let (range, line) = &layout.lines[index];
        let in_line = byte.clamp(range.start, range.end) - range.start;
        point(
            layout.bounds.left() + line.x_for_index(in_line),
            layout.bounds.top() + layout.line_height * (index as f32 + 0.5),
        )
    }

    /// Un solo clic, sin arrastre: el cursor salta al byte pulsado.
    #[gpui::test]
    fn mouse_click_places_the_caret_at_the_clicked_point(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        cx.run_until_parked();
        let at = view.update_in(cx, |view, _window, _cx| {
            point_for_byte(view, EditorTarget::Note, 5)
        });

        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());

        view.update_in(cx, |view, _window, _cx| {
            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.selection, 5..5, "el cursor salta al byte pulsado");
            assert_eq!(edit.cursor(), 5);
            assert!(!view.is_selecting, "el botón ya está levantado");
        });
    }

    /// Arrastre: se selecciona el rango entre el ancla (punto del `mouse_down`)
    /// y la posición actual, sin importar la dirección.
    #[gpui::test]
    fn mouse_drag_selects_the_range_between_anchor_and_position(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        cx.run_until_parked();
        let (anchor, far) = view.update_in(cx, |view, _window, _cx| {
            (
                point_for_byte(view, EditorTarget::Note, 0),
                point_for_byte(view, EditorTarget::Note, 5),
            )
        });

        cx.simulate_mouse_down(anchor, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(far, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(far, MouseButton::Left, Modifiers::default());

        view.update_in(cx, |view, _window, _cx| {
            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.selection, 0..5);
            assert_eq!(edit.selected_text(), "hola ");
            assert!(!view.is_selecting, "el botón ya está levantado");
        });
    }

    /// Shift+clic extiende la selección desde el ancla que dejó el clic previo.
    #[gpui::test]
    fn shift_click_extends_the_selection_from_the_previous_anchor(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        cx.run_until_parked();
        let (anchor, far) = view.update_in(cx, |view, _window, _cx| {
            (
                point_for_byte(view, EditorTarget::Note, 5),
                point_for_byte(view, EditorTarget::Note, 10),
            )
        });

        // Clic simple: coloca el ancla en el byte 5.
        cx.simulate_mouse_down(anchor, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(anchor, MouseButton::Left, Modifiers::default());
        // Shift+clic: extiende hasta el byte 10.
        cx.simulate_mouse_down(
            far,
            MouseButton::Left,
            Modifiers {
                shift: true,
                ..Modifiers::default()
            },
        );
        cx.simulate_mouse_up(far, MouseButton::Left, Modifiers::default());

        view.update_in(cx, |view, _window, _cx| {
            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.selection, 5..10);
            assert_eq!(edit.selected_text(), "mundo");
        });
    }

    /// Navegar con el ratón no ensucia la nota: `commit_selection` no marca
    /// `dirty` (si no, seleccionar dispararía el autoguardado).
    #[gpui::test]
    fn mouse_selection_does_not_dirty_the_note(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        cx.run_until_parked();
        let (anchor, far) = view.update_in(cx, |view, _window, _cx| {
            (
                point_for_byte(view, EditorTarget::Note, 0),
                point_for_byte(view, EditorTarget::Note, 5),
            )
        });

        cx.simulate_mouse_down(anchor, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(far, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(far, MouseButton::Left, Modifiers::default());

        view.update_in(cx, |view, _window, cx| {
            let notes = &view.deck.read(cx).notes;
            assert!(!notes.dirty, "seleccionar con el ratón no ensucia la nota");
            assert_eq!(notes.cursor, 5, "el cursor se vuelca en `Notes`");
            assert_eq!(notes.edit_buffer, "hola mundo", "el texto no cambia");
        });
    }

    /// El flag de arrastre se levanta tanto con `mouse_up` dentro como fuera del
    /// editor (`mouse_up_out`).
    #[gpui::test]
    fn mouse_up_and_mouse_up_out_clear_the_dragging_flag(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        cx.run_until_parked();
        let at = view.update_in(cx, |view, _window, _cx| {
            point_for_byte(view, EditorTarget::Note, 3)
        });

        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
        view.update_in(cx, |view, _window, _cx| {
            assert!(view.is_selecting, "el arrastre queda abierto al pulsar");
        });
        cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
        view.update_in(cx, |view, _window, _cx| {
            assert!(!view.is_selecting, "`mouse_up` cierra el arrastre");
        });

        // Segundo arrastre que termina fuera del editor: GPUI entrega `mouse_up`
        // al elemento que recibió el `mouse_down` (`on_mouse_up_out`).
        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(
            point(px(5.), px(5.)),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        cx.simulate_mouse_up(
            point(px(5.), px(5.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        view.update_in(cx, |view, _window, _cx| {
            assert!(!view.is_selecting, "`mouse_up_out` cierra el arrastre");
        });
    }

    /// El editor del título responde al ratón igual que el de la nota.
    #[gpui::test]
    fn heading_editor_responds_to_mouse(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Heading, "hola mundo");
        cx.run_until_parked();
        let at = view.update_in(cx, |view, _window, _cx| {
            point_for_byte(view, EditorTarget::Heading, 5)
        });

        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());

        view.update_in(cx, |view, _window, cx| {
            let edit = view.heading_edit.as_ref().expect("editor del título");
            assert_eq!(edit.selection, 5..5);
            assert_eq!(view.deck.read(cx).notes.heading_cursor, 5);
        });
    }

    /// Arrastre inverso: el ancla se queda a la derecha y el cursor en el
    /// extremo opuesto, con la selección marcada como invertida.
    #[gpui::test]
    fn mouse_reverse_drag_selects_and_marks_the_selection_as_reversed(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        cx.run_until_parked();
        let (anchor, far) = view.update_in(cx, |view, _window, _cx| {
            (
                point_for_byte(view, EditorTarget::Note, 10),
                point_for_byte(view, EditorTarget::Note, 5),
            )
        });

        cx.simulate_mouse_down(anchor, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(far, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(far, MouseButton::Left, Modifiers::default());

        view.update_in(cx, |view, _window, _cx| {
            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.selection, 5..10);
            assert!(edit.reversed, "el ancla se mantiene a la derecha");
            assert_eq!(edit.cursor(), 5, "el cursor queda en el extremo opuesto");
            assert_eq!(edit.selected_text(), "mundo");
        });
    }

    /// Un punto sin byte resoluble (`index_for_point` → `None`) no mueve el
    /// cursor, no cambia la selección y no abre un arrastre.
    ///
    /// Se comprueba con el clic end-to-end en el relleno del contenedor (fuera
    /// del rectángulo del editor) y, para no depender del hit-testing de ese
    /// relleno en headless, también llamando a `begin_selection` con un punto
    /// claramente fuera del rectángulo.
    #[gpui::test]
    fn mouse_click_outside_the_editor_leaves_the_caret_untouched(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        cx.run_until_parked();
        let in_padding = view.update_in(cx, |view, _window, _cx| {
            let layout = view.note_layout.as_ref().expect("layout del editor");
            let padding = point(
                layout.bounds.left() - px(4.),
                layout.bounds.top() + layout.line_height / 2.,
            );
            assert_eq!(
                layout.index_for_point(padding),
                None,
                "el relleno del contenedor queda fuera del rectángulo del editor"
            );
            padding
        });

        cx.simulate_mouse_down(in_padding, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(in_padding, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(in_padding, MouseButton::Left, Modifiers::default());

        view.update_in(cx, |view, window, cx| {
            view.begin_selection(
                EditorTarget::Note,
                point(px(-50.), px(-50.)),
                false,
                window,
                cx,
            );

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.selection, 10..10, "el cursor no se mueve");
            assert!(!view.is_selecting, "el clic no abre un arrastre");
            assert_eq!(view.deck.read(cx).notes.cursor, 10);
        });
    }

    /// El arrastre no puede quedarse «pegado»: un `mouse_move` sin botón pulsado
    /// (flag obsoleto porque el `mouse_up` nunca llegó) no extiende la selección,
    /// y salir del modo edición durante un arrastre limpia el flag.
    #[gpui::test]
    fn idle_mouse_move_does_not_extend_a_stale_selection(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        cx.run_until_parked();
        let (at5, at8) = view.update_in(cx, |view, _window, _cx| {
            (
                point_for_byte(view, EditorTarget::Note, 5),
                point_for_byte(view, EditorTarget::Note, 8),
            )
        });

        cx.simulate_mouse_down(at5, MouseButton::Left, Modifiers::default());
        // Flag pegado a propósito: simula un `mouse_up` que nunca llegó.
        view.update_in(cx, |view, _window, _cx| {
            view.is_selecting = true;
        });
        cx.simulate_mouse_move(at8, None, Modifiers::default());
        view.update_in(cx, |view, _window, _cx| {
            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(
                edit.selection,
                5..5,
                "un hover sin botón pulsado no extiende la selección"
            );
        });

        // Salir de edición durante un arrastre (Esc, guardar, cancelar) tampoco
        // puede dejarlo abierto: el `mouse_up` posterior ya no llegará.
        cx.simulate_mouse_down(at8, MouseButton::Left, Modifiers::default());
        view.update_in(cx, |view, _window, cx| {
            view.deck.update(cx, |deck, cx| {
                deck.notes.editing = false;
                cx.notify();
            });
        });
        cx.run_until_parked();
        view.update_in(cx, |view, _window, _cx| {
            assert!(!view.is_selecting, "salir de edición limpia el arrastre");
        });
    }

    // -----------------------------------------------------------------------
    // (d) Portapapeles: copiar (Ctrl+C) y cortar (Ctrl+X).
    // -----------------------------------------------------------------------

    #[gpui::test]
    fn ctrl_c_copies_the_selection_without_touching_the_note(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        view.update_in(cx, |view, window, cx| {
            view.set_selected_text_range(0..4, window, cx);
            let event = key_down(
                "c",
                None,
                Modifiers {
                    control: true,
                    ..Default::default()
                },
            );
            assert!(
                view.handle_edit_key(&event, window, cx),
                "Ctrl+C se consume"
            );

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.text, "hola mundo", "copiar no toca el texto");
            assert_eq!(edit.selection, 0..4, "la selección se conserva");

            let notes = &view.deck.read(cx).notes;
            assert_eq!(notes.edit_buffer, "hola mundo");
            assert!(!notes.dirty, "copiar no ensucia la nota");
        });

        let copied = cx.read_from_clipboard().and_then(|item| item.text());
        assert_eq!(
            copied.as_deref(),
            Some("hola"),
            "el portapapeles lleva la selección"
        );
    }

    #[gpui::test]
    fn ctrl_x_cuts_the_selection_and_marks_the_note_dirty(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola mundo");
        view.update_in(cx, |view, window, cx| {
            view.set_selected_text_range(0..5, window, cx);
            let event = key_down(
                "x",
                None,
                Modifiers {
                    control: true,
                    ..Default::default()
                },
            );
            assert!(
                view.handle_edit_key(&event, window, cx),
                "Ctrl+X se consume"
            );

            let edit = view.note_edit.as_ref().expect("editor de la nota");
            assert_eq!(edit.text, "mundo", "cortar borra la selección");
            assert_eq!(
                edit.selection,
                0..0,
                "el cursor queda donde estaba el corte"
            );

            let notes = &view.deck.read(cx).notes;
            assert_eq!(notes.edit_buffer, "mundo");
            assert_eq!(notes.cursor, 0);
            assert!(notes.dirty, "cortar sí cambia el texto");
        });

        let cut = cx.read_from_clipboard().and_then(|item| item.text());
        assert_eq!(cut.as_deref(), Some("hola "), "cortar copia la selección");
    }

    #[gpui::test]
    fn ctrl_x_cuts_the_selection_in_the_heading(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Heading, "Titulo");
        view.update_in(cx, |view, window, cx| {
            view.set_selected_text_range(0..3, window, cx);
            let event = key_down(
                "x",
                None,
                Modifiers {
                    control: true,
                    ..Default::default()
                },
            );
            assert!(
                view.handle_edit_key(&event, window, cx),
                "Ctrl+X se consume"
            );

            let edit = view.heading_edit.as_ref().expect("editor del título");
            assert_eq!(edit.text, "ulo", "cortar borra la selección del título");
            assert_eq!(edit.selection, 0..0);

            let notes = &view.deck.read(cx).notes;
            assert_eq!(
                notes.heading_edit_buffer, "ulo",
                "el corte se vuelca al buffer del título"
            );
            assert_eq!(notes.heading_cursor, 0);
            // El título se persiste con Enter (`save_heading`), así que escribir
            // en él no marca la nota como sucia: es intencional.
            assert!(notes.heading_editing, "el editor del título sigue activo");
        });

        let cut = cx.read_from_clipboard().and_then(|item| item.text());
        assert_eq!(cut.as_deref(), Some("Tit"), "cortar copia la selección");
    }

    #[gpui::test]
    fn ctrl_c_without_a_selection_is_a_noop(cx: &mut TestAppContext) {
        let (view, cx) = presenter_editing(cx, EditorTarget::Note, "hola");
        // Contenido previo: un no-op no debe sobrescribirlo.
        cx.write_to_clipboard(gpui::ClipboardItem::new_string("previo".to_string()));

        view.update_in(cx, |view, window, cx| {
            let event = key_down(
                "c",
                None,
                Modifiers {
                    control: true,
                    ..Default::default()
                },
            );
            assert!(
                view.handle_edit_key(&event, window, cx),
                "Ctrl+C se consume aunque no haya selección"
            );

            assert_eq!(
                view.note_edit
                    .as_ref()
                    .expect("editor de la nota")
                    .selection,
                4..4,
                "sin selección la selección queda colapsada"
            );
            assert!(
                !view.deck.read(cx).notes.dirty,
                "copiar sin selección no ensucia la nota"
            );
        });

        let kept = cx.read_from_clipboard().and_then(|item| item.text());
        assert_eq!(
            kept.as_deref(),
            Some("previo"),
            "el portapapeles no se ha tocado"
        );
    }
}
