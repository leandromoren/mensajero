use eframe::egui;
use reqwest::Client;
use std::collections::HashMap;
use anyhow::Error;
use egui_dock::{DockArea, DockState, TabViewer};

#[derive(PartialEq, Clone)] 
enum Tab {
    Headers,
    Body,
    Authorization,
}

struct Mensajero {
    url: String,
    method: String,
    response: Option<String>,
    request_body: Option<String>,
    client: Client,
    headers: HashMap<String, String>,
    status_code: String,
    dock_state: DockState<Tab>,
    authentication: Option<String>,
}

impl Default for Mensajero {
    fn default() -> Self {
        let tabs = vec![
            Tab::Headers,
            Tab::Body,
            Tab::Authorization,
        ];

        let dock_state = DockState::new(tabs);

        Self {
            url: String::new(),
            method: "GET".to_string(),
            response: None,
            request_body: None,
            client: Client::new(),
            headers: HashMap::new(),
            status_code: String::new(),
            dock_state,
            authentication: None,
        }
    }
}
impl Mensajero {
    fn send_request(&mut self) {
        // Clona la URL y el método para usarlos en la solicitud
        let url = self.url.clone();
        let method = self.method.clone();
        let client = &self.client;

        // Establece un cuerpo de solicitud vacío si no se ha proporcionado uno
        let body = self.request_body.clone().unwrap_or_default();

        // Realiza la solicitud asíncrona
        let response: Result<reqwest::Response, Error> = tokio::runtime::Runtime::new().unwrap().block_on(async {
            match method.as_str() {
                "GET" => client.get(&url).send().await.map_err(Error::from),
                "POST" => client.post(&url).body(body).send().await.map_err(Error::from),
                "PUT" => client.put(&url).body(body).send().await.map_err(Error::from),
                "DELETE" => client.delete(&url).send().await.map_err(Error::from),
                _ => Err(Error::msg("Método no válido")),
            }
        });

        // Maneja la respuesta
        match response {
            Ok(resp) => {
                self.status_code = resp.status().to_string();
                self.response = Some(
                    tokio::runtime::Runtime::new().unwrap().block_on(resp.text()).unwrap_or_else(|_| "Error al obtener el cuerpo de la respuesta".to_string())
                );
            }
            Err(err) => {
                self.status_code = "Error".to_string();
                self.response = Some(format!("Error: {}", err));
            }
        }
    }
}

impl TabViewer for Mensajero {
    type Tab = Tab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Headers => {
                ui.horizontal(|ui| {
                        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
                        self.headers.insert("Accept".to_string(), "application/json".to_string());
                        self.headers.insert("Authorization".to_string(), "Bearer".to_string());
                        self.headers.insert("User-Agent".to_string(), "Mensajero v0.1".to_string());
                        self.headers.insert("Host".to_string(), "localhost:8080".to_string());
                        self.headers.insert("Connection".to_string(), "keep-alive".to_string());
                        self.headers.insert("Cache-Control".to_string(), "no-cache".to_string());
                });

                let keys: Vec<String> = self.headers.keys().cloned().collect();
                for key in keys {
                    if let Some(value) = self.headers.get_mut(&key) {
                        ui.horizontal(|ui| {
                            ui.label(&key);
                            ui.text_edit_singleline(value);
                        });
                    }
                }
            }
            Tab::Body => {
                ui.heading("Body");
                if self.method == "POST" || self.method == "PUT" {
                    if self.request_body.is_none() {
                        self.request_body = Some(String::new());
                    }
                    if let Some(body) = &mut self.request_body {
                        ui.text_edit_multiline(body);
                    }
                } else {
                    ui.label("Body disponible solo para métodos POST o PUT");
                }
            }
            Tab::Authorization => {
                ui.heading("Authentication");
                ui.horizontal(|ui| {
                    ui.label("Token:");
                    if self.authentication.is_none() {
                        self.authentication = Some(String::new());
                    }
                    if let Some(auth) = &mut self.authentication {
                        ui.text_edit_singleline(auth);
                    }
                });
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Headers => "Headers".into(),
            Tab::Body => "Body".into(),
            Tab::Authorization => "Authorization".into(),
        }
    }
}

impl eframe::App for Mensajero {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Coloca todos los elementos dentro de una columna
            ui.vertical(|ui| {
                // URL input
                ui.horizontal(|ui| {
                    let style = ui.style_mut();
                    style.spacing.button_padding = egui::vec2(10.0, 10.0); // Ajusta el padding del botón
                    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(200);

                    // ComboBox para seleccionar el método HTTP
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
                        self.send_request();
                    }
                });

                ui.separator();

                // Ahora coloca los tabs debajo del formulario
                DockArea::new(&mut self.dock_state.clone())
                    .show_inside(ui, self);
            });
        });
    }
}


fn main() {
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
