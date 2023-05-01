#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use mouse_position::mouse_position::Mouse;

fn get_mouse_pos() -> (i32, i32) {
    let position = Mouse::get_mouse_position();
    match position {
        Mouse::Position { x, y } => (x,y),
        Mouse::Error => panic!("Error getting mouse position"),
   }
}

fn calculate_euclidean_distance(start: &(i32,i32), end: &(i32,i32)) -> f64 {
    let x = (end.0 - start.0).pow(2);
    let y = (end.1 - start.1).pow(2);
    ((x + y) as f64).sqrt()
}

fn calculate_dpi_deviation(measured_dpi: f64, set_dpi: f64) -> f64 {
    ((measured_dpi / set_dpi) - 1.0) * 100.0
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("DPI Checker", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));
}

struct MyEguiApp {

    dpi_str: String,
    distance_str: String,

    dpi: f32,
    distance: f32,

    error_message: String,

    starting_global_pos: (i32, i32),
    starting_pos_set: bool,

    starting_local_pos: egui::Pos2,

    measured_dpi: f64,
    dpi_deviation: f64
}



impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            dpi_str: "400".to_string(),
            distance_str: "1".to_string(),
            dpi: 400.0,
            distance: 1.0,
            error_message: String::new(),
            starting_global_pos: (0,0),
            starting_pos_set: false,
            starting_local_pos: egui::Pos2::default(),
            measured_dpi: 0.0,
            dpi_deviation: 0.0
        }
    }
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        egui::SidePanel::left("settings_panel")
        .min_width(100.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Settings");
            ui.add_space(20.0);
            ui.label("DPI");
            ui.add(egui::TextEdit::singleline(&mut self.dpi_str));
            ui.label("Distance (inches)");
            ui.add(egui::TextEdit::singleline(&mut self.distance_str));
            ui.add_space(10.0);
            if ui.button("Apply").clicked() {
                self.error_message.clear();
                match self.distance_str.parse::<f32>() {
                    Ok(distance) => self.distance = distance,
                    Err(_) => self.error_message += "Distance must be a number\n\n"
                }
                match self.dpi_str.parse::<f32>() {
                    Ok(dpi) => self.dpi = dpi,
                    Err(_) => self.error_message += "DPI must be a number\n\n"}
            }
            if !self.error_message.is_empty() {
                ui.add_space(10.0);
                ui.label(self.error_message.clone());
            }
            ui.add_space(40.0);
            ui.label(format!("Measured DPI: {:.2}", self.measured_dpi));
            ui.label(format!("DPI Deviation: {:.2}%", self.dpi_deviation));
            
        });
        let my_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            outer_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            rounding: egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 0.0 },
            shadow: eframe::epaint::Shadow::NONE,
            fill: egui::Color32::LIGHT_BLUE,
            stroke: egui::Stroke::NONE,
        };
        egui::CentralPanel::default().frame(my_frame).show(ctx, |ui| {
            let plot = egui::plot::Plot::new("my_plot")
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .center_x_axis(false)
                .center_y_axis(false)
                .show_x(false)
                .show_y(false);

            let mut local_screen_coordinates = egui::Pos2::new(0.0, 0.0);
            let plot_inner_response = plot.show(ui, |ui| {
                match ui.pointer_coordinate() {
                    Some(point) => {
                        local_screen_coordinates = ui.screen_from_plot(point);
                    },
                    _ => {}
                }
            });

            let response = &plot_inner_response.response;

            if response.hovered() {
                egui::show_tooltip(ctx, egui::Id::new("coordinates_tooltip"), |ui| {
                    ui.label(format!("x: {:.1}, y: {:.1}", local_screen_coordinates.x, local_screen_coordinates.y));
                });
                let global_screen_coordinates = get_mouse_pos();

                if !self.starting_pos_set {
                    if response.clicked() {
                        self.starting_global_pos = global_screen_coordinates;
                        self.starting_pos_set = true;
                        self.starting_local_pos = local_screen_coordinates;
                        
                    }
                } else {
                    if response.clicked() {
                        self.starting_pos_set = false;
                        let distance_moved = calculate_euclidean_distance(&self.starting_global_pos, &global_screen_coordinates);
                        self.measured_dpi = distance_moved / (self.distance as f64);
                        self.dpi_deviation = calculate_dpi_deviation(self.measured_dpi, self.dpi as f64);
                    } else {
                        ui.painter().line_segment(
                            [egui::Pos2::new(self.starting_local_pos.x, self.starting_local_pos.y),
                            egui::Pos2::new(local_screen_coordinates.x, local_screen_coordinates.y)],
                            (1.0, egui::Color32::WHITE));

                    }
                }
            }
        });
   }
}