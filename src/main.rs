use eframe::egui;
use reqwest::Client;
use std::collections::HashMap;

// Defino la estructura de la aplicación
struct Mensajero{
    url: String,
    method: String,
    response: Option<String>,
    request_body: Option<String>,
    client: Client,
    headers: HashMap<String,String>,
    status_code: String,
}

impl Default for Mensajero {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: "GET".to_string(),
            response:None,
            request_body:None,
            client:Client::new(),
            headers:HashMap::new(),
            status_code:String::new(),
        }
    }
}

impl eframe::App for Mensajero {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.heading("Mensajero");

             // URL input
            ui.horizontal(|ui| {
                let style = ui.style_mut();
                style.spacing.button_padding = egui::vec2(10.0, 10.0); // Ajusta el padding del botón
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(200);
                egui::ComboBox::from_label("")
                    .selected_text(&self.method)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.method, "GET".to_string(), "GET");
                        ui.selectable_value(&mut self.method, "POST".to_string(), "POST");
                        ui.selectable_value(&mut self.method, "PUT".to_string(), "PUT");
                        ui.selectable_value(&mut self.method, "DELETE".to_string(), "DELETE");
                    });
                
                let available_width = ui.available_width() - 120.0;
                egui::TextEdit::singleline(&mut self.url)
                    .hint_text("URL:")
                    .desired_width(available_width)
                    .font(egui::TextStyle::Monospace)
                    .min_size(egui::vec2(100.0, 35.0))
                    .font(egui::TextStyle::Heading)
                    .text_color(egui::Color32::from_rgb(255, 255, 0))
                    .show(ui);

               if ui.add(egui::Button::new(
                    egui::RichText::new("Enviar").size(13.0))
                    .min_size(egui::vec2(100.0, 10.0))
                ).clicked() {
                    //self.send_request();
                }
            });

            // Response area
            if let Some(response) = &self.response {
                ui.label("Response:");
                ui.text_edit_multiline(&mut response.clone());
            }
        });
    }
}

fn main () {
   let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 720.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Mensajero v0.1",
        options,
        Box::new(|_cc| Ok(Box::new(Mensajero::default()))),
    ).unwrap();

}