use egui::{Context, Visuals, Color32, CornerRadius, Stroke, Vec2, style::WidgetVisuals};

fn headerbar_bg(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(48, 48, 48) } else { Color32::from_rgb(235, 235, 235) }
}

fn window_bg(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(36, 36, 36) } else { Color32::from_rgb(250, 250, 250) }
}

fn view_bg(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(30, 30, 30) } else { Color32::from_rgb(255, 255, 255) }
}

fn text_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(255, 255, 255) } else { Color32::from_rgb(0, 0, 0) }
}

fn border_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgba_premultiplied(255, 255, 255, 25) } else { Color32::from_rgba_premultiplied(0, 0, 0, 18) }
}

fn code_bg(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(42, 42, 42) } else { Color32::from_rgb(244, 244, 244) }
}

const CORNER: CornerRadius = CornerRadius::same(6);
const ACCENT: Color32 = Color32::from_rgb(53, 132, 228);

pub fn apply_gtk4_style(ctx: &Context, dark_mode: bool) {
    let dark = dark_mode;

    let mut visuals = if dark { Visuals::dark() } else { Visuals::light() };

    visuals.window_fill = window_bg(dark);
    visuals.panel_fill = headerbar_bg(dark);
    visuals.extreme_bg_color = code_bg(dark);
    visuals.code_bg_color = code_bg(dark);
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::NONE;
    visuals.hyperlink_color = ACCENT;

    visuals.widgets.noninteractive = WidgetVisuals {
        bg_fill: view_bg(dark),
        weak_bg_fill: Color32::TRANSPARENT,
        bg_stroke: Stroke::new(1.0, border_color(dark)),
        fg_stroke: Stroke::new(1.0, text_color(dark)),
        corner_radius: CORNER,
        expansion: 0.0,
    };

    visuals.widgets.inactive = WidgetVisuals {
        bg_fill: Color32::TRANSPARENT,
        weak_bg_fill: Color32::TRANSPARENT,
        bg_stroke: Stroke::NONE,
        fg_stroke: Stroke::new(1.0, text_color(dark)),
        corner_radius: CORNER,
        expansion: 0.0,
    };

    let hover_fill = if dark {
        Color32::from_rgba_premultiplied(53, 132, 228, 51)
    } else {
        Color32::from_rgba_premultiplied(53, 132, 228, 31)
    };

    visuals.widgets.hovered = WidgetVisuals {
        bg_fill: hover_fill,
        weak_bg_fill: Color32::TRANSPARENT,
        bg_stroke: Stroke::NONE,
        fg_stroke: Stroke::new(1.0, ACCENT),
        corner_radius: CORNER,
        expansion: 0.0,
    };

    visuals.widgets.active = WidgetVisuals {
        bg_fill: ACCENT,
        weak_bg_fill: Color32::TRANSPARENT,
        bg_stroke: Stroke::NONE,
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        corner_radius: CORNER,
        expansion: 0.0,
    };

    visuals.widgets.open = WidgetVisuals {
        bg_fill: if dark { Color32::from_rgb(60, 60, 60) } else { Color32::from_rgb(230, 230, 230) },
        weak_bg_fill: Color32::TRANSPARENT,
        bg_stroke: Stroke::new(1.0, border_color(dark)),
        fg_stroke: Stroke::new(1.0, text_color(dark)),
        corner_radius: CORNER,
        expansion: 0.0,
    };

    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(8.0, 4.0);
        style.spacing.indent = 16.0;
        style.spacing.scroll.bar_width = 6.0;
        style.spacing.scroll.bar_outer_margin = 0.0;
        style.spacing.interact_size = Vec2::new(0.0, 34.0);
    });
}