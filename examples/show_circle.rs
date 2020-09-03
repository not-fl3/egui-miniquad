use macroquad::*;
use egui::{Layout, Align};

#[macroquad::main("Simple Macroquad Egui")]
async fn main() {
    let mut ui = emigui_miniquad::UiPlugin::new();
    let mut circle_size = 50.0;
    let mut show_circle = true;

    loop {
        clear_background(WHITE);

        ui.macroquad(|ui| {
            egui::Window::new("Circle").show(ui.ctx(), |ui| {
                if ui.button(if show_circle { "Hide" } else { "Show" }).clicked {
                    show_circle = !show_circle;
                }
                ui.add(egui::DragValue::f32(&mut circle_size));
            });
        });

        if show_circle {
            draw_circle(screen_width() / 2.0, screen_height() / 2.0, circle_size, RED);
        }

        next_frame().await;
    }
}
