use anyhow::Error;
use eframe::egui;
use egui_dock::{DockArea, DockState, TabViewer};
use reqwest::Client;
use std::collections::HashMap;
use std::vec::Vec;

#[derive(PartialEq, Clone)]
pub enum Tab {
    Params,
    Headers,
    Body,
    Authorization,
}

#[derive(PartialEq)]
pub enum BodyScreen {
    Request,
    Response,
}

pub struct Mensajero {
    url: String,
    method: String,
    response: Option<String>,
    request_body: Option<String>,
    client: Client,
    headers: HashMap<String, String>,
    status_code: String,
    dock_state: DockState<Tab>,
    authentication: Option<String>,
    params: Vec<(String, String, String)>,
    body_screen: BodyScreen,
}

impl Default for BodyScreen {
    fn default() -> Self {
        BodyScreen::Request
    }
}

impl Default for Mensajero {
    fn default() -> Self {
        let tabs = vec![Tab::Params, Tab::Headers, Tab::Body, Tab::Authorization];

        let dock_state = DockState::new(tabs);

        // Configurar el cliente con límites más altos
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(0)
            .http1_only()  // Use HTTP/1.1 which is more stable for large responses
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            url: String::new(),
            method: "GET".to_string(),
            response: None,
            request_body: None,
            client,
            headers: HashMap::new(),
            status_code: String::new(),
            dock_state,
            authentication: None,
            params: vec![("".to_string(), "".to_string(), "".to_string()); 4],
            body_screen: BodyScreen::default(),
        }
    }
}
impl Mensajero {
    fn update_url_with_params(&mut self) {
        // Obtener la base de la URL sin parámetros (todo lo antes de '?')
        let base_url = if let Some(pos) = self.url.find('?') {
            &self.url[..pos]
        } else {
            &self.url
        }
        .to_string();
    
        // Iniciar una nueva URL basada en la base sin parámetros
        let mut url_with_params = base_url.clone();
    
        // Añadir los parámetros clave y valor.
        let mut is_first = true;
        for (key, value, _) in &self.params {
            // Mostrar el parámetro incluso si solo la clave está llena
            if !key.is_empty() {
                if is_first {
                    url_with_params.push('?');
                    is_first = false;
                } else {
                    url_with_params.push('&');
                }
    
                // Agregar solo clave si el valor está vacío
                if value.is_empty() {
                    url_with_params.push_str(&format!("{}", key));
                } else {
                    url_with_params.push_str(&format!("{}={}", key, value));
                }
            }
        }
    
        // Actualizar la URL si hay un cambio
        if self.url != url_with_params {
            self.url = url_with_params;
        }
    }

    fn send_request(&mut self) {
        let url = self.url.clone();
        let method = self.method.clone();
        let client = &self.client;
        let body = self.request_body.clone().unwrap_or_default();

        // Create a single runtime instance outside the response handling
        let rt = tokio::runtime::Runtime::new().unwrap();

        let response: Result<reqwest::Response, Error> = rt.block_on(async {
            let request = match method.as_str() {
                "GET" => client.get(&url),
                "POST" => client.post(&url).body(body),
                "PUT" => client.put(&url).body(body),
                "DELETE" => client.delete(&url),
                _ => return Err(Error::msg("Método no válido")),
            };

            request
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .send()
                .await
                .map_err(Error::from)
        });

        match response {
            Ok(resp) => {
                self.status_code = resp.status().to_string();
                
                // Crear un nuevo runtime para la operación asíncrona
                let rt = tokio::runtime::Runtime::new().unwrap();
                
                // Obtener el cuerpo completo de la respuesta usando bytes()
                let response_text = rt.block_on(async {
                    match resp.bytes().await {
                        Ok(bytes) => {
                            match String::from_utf8(bytes.to_vec()) {
                                Ok(text) => {
                                    // Intentar formatear como JSON si es posible
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                        serde_json::to_string_pretty(&json).unwrap_or(text)
                                    } else {
                                        text
                                    }
                                },
                                Err(e) => format!("Error al convertir bytes a texto: {}", e)
                            }
                        }
                        Err(e) => format!("Error al obtener el cuerpo de la respuesta: {}", e),
                    }
                });

                // Imprimir información de diagnóstico
                println!("Status Code: {}", self.status_code);
                println!("Response length: {} bytes", response_text.len());
                
                // Guardar la respuesta completa
                self.response = Some(response_text);
            }
            Err(err) => {
                self.status_code = "Error".to_string();
                self.response = Some(format!("Error en la solicitud: {}", err));
            }
        }
    }
}

impl TabViewer for Mensajero {
    type Tab = Tab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Params => {
                ui.heading("Query params");

               // Variable para saber si hubo algún cambio
               let mut changed = false;

                // Mostrar cada parámetro con campos de texto para clave y valor
                for (_i, (key, value, description)) in self.params.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        // Campo para la clave
                        let key_response = ui.add(
                            egui::TextEdit::singleline(key)
                                .hint_text("Clave")
                                .min_size(egui::vec2(100.0, 35.0))
                                .font(egui::TextStyle::Heading),
                        );

                        ui.label("=");

                        // Campo para el valor
                        let value_response = ui.add(
                            egui::TextEdit::singleline(value)
                                .hint_text("Valor")
                                .min_size(egui::vec2(100.0, 35.0))
                                .font(egui::TextStyle::Heading),
                        );

                        ui.label("=");

                        // Campo para la descripción
                        ui.add(
                            egui::TextEdit::singleline(description)
                                .hint_text("Descripción")
                                .min_size(egui::vec2(100.0, 35.0))
                                .font(egui::TextStyle::Heading),
                        );

                        if key_response.changed() || value_response.changed() {
                            changed = true;
                        }

                    });
                }
                // Actualizar la URL cada vez que se modifique un parámetro
                if changed {
                    self.update_url_with_params();
                    ui.ctx().request_repaint();
                }
            }

            Tab::Headers => {
                ui.horizontal(|_ui| {
                    self.headers
                        .insert("Content-Type".to_string(), "application/json".to_string());
                    self.headers
                        .insert("Accept".to_string(), "application/json".to_string());
                    self.headers
                        .insert("Authorization".to_string(), "Bearer".to_string());
                    self.headers
                        .insert("User-Agent".to_string(), "Mensajero v0.1".to_string());
                    self.headers
                        .insert("Host".to_string(), "localhost:8080".to_string());
                    self.headers
                        .insert("Connection".to_string(), "keep-alive".to_string());
                    self.headers
                        .insert("Cache-Control".to_string(), "no-cache".to_string());
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
            
                ui.vertical(|ui| {
                    let available_height = ui.available_height();
                    let request_height = (available_height * 0.5).max(250.0); // 50% del espacio disponible
                    let response_height = (available_height * 0.5).max(500.0); // 100% del espacio disponible
                    
                    ui.label("JSON Request:");
                    if self.request_body.is_none() {
                        self.request_body = Some(String::new());
                    }
                    if let Some(request) = &mut self.request_body {
                        egui::ScrollArea::vertical()
                            .id_salt("request_scroll") // ID único para el scroll del request
                            .max_height(request_height)
                            .show(ui, |ui| {
                                ui.push_id("request_edit", |ui| { // ID único para el TextEdit del request
                                    ui.add(
                                        egui::TextEdit::multiline(request)
                                            .hint_text("Escribe tu solicitud en formato JSON aquí")
                                            .desired_width(ui.available_width())
                                            .desired_rows(28)

                                    )
                                });
                            });
                    }
                    ui.add_space(20.0); // Añade 20 píxeles de espacio vertical
                    ui.separator();
                    ui.add_space(20.0);
                    if let Some(response) = &self.response {
                        egui::ScrollArea::vertical()
                            .id_salt("response_scroll") // ID único para el scroll del response
                            .max_height(response_height)
                            .show(ui, |ui| {
                                ui.push_id("response_edit", |ui| { // ID único para el TextEdit del response
                                    ui.add(
                                        egui::TextEdit::multiline(&mut response.clone())
                                            .desired_width(ui.available_width())
                                            .desired_rows(28)
                                            .font(egui::TextStyle::Monospace)
                                            .interactive(true)
                                    )
                                });
                            });
                    } 
                });
            },
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
            Tab::Params => "Params".into(),
            Tab::Headers => "Headers".into(),
            Tab::Body => "Body".into(),
            Tab::Authorization => "Authorization".into(),
        }
    }
}

impl eframe::App for Mensajero {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            let mut style = (*ctx.style()).clone();
            
            // Paleta base
            let window_fill = egui::Color32::from_rgb(255, 255, 255);    // Blanco puro
            let panel_fill = egui::Color32::from_rgb(250, 250, 250);     // Gris casi blanco
            let text_color = egui::Color32::from_rgb(51, 51, 51);        // Gris oscuro
            let hover_color = egui::Color32::from_rgb(245, 245, 245);    // Gris muy claro
            let border_color = egui::Color32::from_rgb(230, 230, 230);   // Gris claro para bordes
            
            // Colores para botones
            let button_color = egui::Color32::from_rgb(255, 149, 0);     // Naranja principal
            let button_hover = egui::Color32::from_rgb(255, 165, 41);    // Naranja más claro para hover
            let button_active = egui::Color32::from_rgb(230, 134, 0);    // Naranja más oscuro para click

            // Configuración general
            style.visuals.window_fill = window_fill;
            style.visuals.panel_fill = panel_fill;
            style.visuals.extreme_bg_color = panel_fill;
            
            // Widgets normales (no botones)
            style.visuals.widgets.noninteractive.bg_fill = window_fill;
            style.visuals.widgets.noninteractive.fg_stroke.color = text_color;
            style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border_color);
            
            // Botones
            style.visuals.widgets.inactive.bg_fill = button_color;
            style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::BLACK; // Texto blanco en botones
            style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
            
            // Hover de botones
            style.visuals.widgets.hovered.bg_fill = button_hover;
            style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::BLACK;
            style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
            
            // Click en botones
            style.visuals.widgets.active.bg_fill = button_active;
            style.visuals.widgets.active.fg_stroke.color = egui::Color32::BLACK;
            style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
            
            // Bordes redondeados
            style.visuals.widgets.noninteractive.rounding = 4.0.into();
            style.visuals.widgets.inactive.rounding = 4.0.into();
            style.visuals.widgets.hovered.rounding = 4.0.into();
            style.visuals.widgets.active.rounding = 4.0.into();
            
            // Selección
            style.visuals.selection.bg_fill = button_color.linear_multiply(0.2);
            
            // Espaciado
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.window_margin = egui::Margin::same(16.0);
            style.spacing.button_padding = egui::vec2(12.0, 8.0);

            ctx.set_style(style);
        }
        egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(10.0))
        .show(ctx, |ui| {

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
                        .min_size(egui::vec2(100.0, 35.0))
                        .font(egui::TextStyle::Heading)
                        .text_color(egui::Color32::from_rgb(255, 255, 0))
                        .show(ui);

                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Enviar").size(13.0))
                                .min_size(egui::vec2(100.0, 10.0)),
                        )
                        .clicked()
                    {
                        self.send_request();
                        if let Some(body) = &self.request_body {
                            println!("Request: {}", body);
                        }
                    }
                });

                ui.separator();
                // Clone the dock_state instead of moving it
                let mut dock_state = self.dock_state.clone();
                DockArea::new(&mut dock_state).show_inside(ui, self);
                // Update the original dock_state with the modified one
                self.dock_state = dock_state;
            });
        });
    }
}


fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1360.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mensajero v0.1",
        options,
        Box::new(|_cc| Ok(Box::new(Mensajero::default()))),
    )
    .unwrap();
}
