use egui::{Context, Visuals, Color32, CornerRadius};

pub fn apply_gtk4_style(ctx: &Context, dark_mode: bool) {
    let mut visuals = if dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    // Redondeo estilo Adwaita/GTK4 (6px)
    let corner_radius = CornerRadius::same(6);
    visuals.widgets.noninteractive.corner_radius = corner_radius;
    visuals.widgets.inactive.corner_radius = corner_radius;
    visuals.widgets.hovered.corner_radius = corner_radius;
    visuals.widgets.active.corner_radius = corner_radius;
    visuals.widgets.open.corner_radius = corner_radius;

    // Colores base
    if dark_mode {
        visuals.window_fill = Color32::from_rgb(36, 36, 36); // Dark Adwaita background
        visuals.panel_fill = Color32::from_rgb(30, 30, 30);
    } else {
        visuals.window_fill = Color32::from_rgb(250, 250, 250); // Light Adwaita background
        visuals.panel_fill = Color32::from_rgb(246, 246, 246);
    }

    // Acento Azul GTK4 (#3584e4)
    let accent_blue = Color32::from_rgb(53, 132, 228);
    visuals.selection.bg_fill = accent_blue;
    
    // Botones inactivos un poco más integrados
    visuals.widgets.inactive.bg_fill = if dark_mode {
        Color32::from_rgb(45, 45, 45)
    } else {
        Color32::from_rgb(235, 235, 235)
    };

    ctx.set_visuals(visuals);
}
