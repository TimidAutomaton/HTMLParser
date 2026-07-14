use std::fs;
use serde::{Deserialize, Serialize};


use crate::html_parser::{self, lexer::LexerToken, token_parser::HTMLTagToken};



#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum TabPages {
    Lexer,
    Parser,
    Tree
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    html_file: String,

    current_token: i32,

    #[serde(skip)]
    lexer_tokens: Vec<LexerToken>,

    #[serde(skip)]
    parser_tokens: Vec<HTMLTagToken>,

    #[serde(skip)]
    tag_strings: Vec<(usize, String)>,

    tab_page: TabPages,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,

            html_file: "".to_string(),
            lexer_tokens: Vec::new(),
            parser_tokens: Vec::new(),
            tag_strings: Vec::new(),

            current_token: 0,
            tab_page: TabPages::Lexer,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.


        let html_file = fs::read_to_string("./assets/html_test.html")
            .expect("Should have been able to read the file");
        let mut lexer = html_parser::lexer::HTMLLexer::new();
        let lexer_tokens = lexer.lex_file(&html_file);

        let mut parser = html_parser::token_parser::TokenParser::new();
        let parser_tokens = parser.parse_tokens(&lexer_tokens);
        
        let mut tree_builder = html_parser::tree_builder::HTMLTreeBuilder::new();
        let tag_tree = tree_builder.build_tree(&parser_tokens);
        let tag_strings = html_parser::tree_builder::get_tree_string(0, &tag_tree);

        /* 
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
        */

        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            html_file,
            lexer_tokens,
            parser_tokens,
            tag_strings,
            current_token: 0,
            tab_page: TabPages::Lexer,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
           


            ui.horizontal(|ui| {
                ui.radio_value(&mut self.tab_page, TabPages::Lexer, "Lexer");
                ui.radio_value(&mut self.tab_page, TabPages::Parser, "Parser");
                ui.radio_value(&mut self.tab_page, TabPages::Tree, "Tree");
            });

            ui.horizontal(|ui| {
                if ui.button("<").clicked() {
                    self.current_token -= 1;
                }

                if ui.button(">").clicked() {
                    self.current_token += 1;
                }
            });

            match self.tab_page {
                TabPages::Lexer => {
                    if self.current_token < 0 {
                        self.current_token = self.lexer_tokens.len() as i32 - 1;
                    }

                    if self.current_token >= self.lexer_tokens.len() as i32 {
                        self.current_token = 0;
                    }

                    ui.heading("Lexer");

                    egui::Grid::new("LRColumns").show(ui, |ui| {

                        let token = &self.lexer_tokens[self.current_token as usize];
                        let start = token.index;
                        let mut end = start + token.length;
                        if end > self.html_file.len() {
                            end = self.html_file.len();
                        }
                        
                        let p1 = &self.html_file[0..start];
                        let p2 = &self.html_file[start..end];
                        let p3 = &self.html_file[end..self.html_file.len()];

                        let style = egui::Style::default();
                        let mut layout_job = egui::text::LayoutJob::default();
                        egui::RichText::new(p1).append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                        egui::RichText::new(p2).background_color(egui::Color32::YELLOW).append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                        egui::RichText::new(p3).append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);

                        let label_response = ui.label(layout_job);
                        let label_height = label_response.rect.height();

                        let default_row_height = style.spacing.interact_size.y;

                        let rows = (label_height / default_row_height).floor() as usize;
                        let row_half = rows as i32 / 2;
                        
                        let mut beginning_index: i32 = self.current_token - row_half;
                        let ending_index = self.current_token + rows as i32 - row_half;
                        if beginning_index < 0 {
                            beginning_index = 0;
                        }
                        if ending_index >= self.lexer_tokens.len() as i32{
                            beginning_index = (self.lexer_tokens.len() - rows) as i32;
                        }

                        egui::Grid::new("Grid")
                        .min_col_width(20.0)
                        .striped(true)
                        .show(ui, |ui| {
                            for i in beginning_index as usize..(beginning_index as usize + rows) {
                                let t = &self.lexer_tokens[i];
                                if i == self.current_token as usize {
                                    ui.label(egui::RichText::new(format!("{:4}", i)).background_color(egui::Color32::YELLOW));
                                    ui.label(egui::RichText::new(format!("{:4}", t.position.1)).background_color(egui::Color32::YELLOW));
                                    ui.label(egui::RichText::new(format!("{:4}", t.position.0)).background_color(egui::Color32::YELLOW));
                                    ui.label(egui::RichText::new(format!("{:?}", t.token_type)).background_color(egui::Color32::YELLOW));
                                    ui.label(egui::RichText::new(format!("{}", t.raw_string.escape_debug())).background_color(egui::Color32::YELLOW));
                                }
                                else {
                                    ui.label(egui::RichText::new(format!("{:4}", i)));
                                    ui.label(egui::RichText::new(format!("{:4}", t.position.1)));
                                    ui.label(egui::RichText::new(format!("{:4}", t.position.0)));
                                    ui.label(egui::RichText::new(format!("{:?}", t.token_type)));
                                    ui.label(egui::RichText::new(format!("{}", t.raw_string.escape_debug())));
                                }
                                
                                // ui.label(egui::RichText::new(format!("{:?}", t.position.1)).background_color(egui::Color32::RED));
                                // println!("{} {}", t.position.1, t.position.0);
                                ui.end_row();
                            }
                            
                        });
                       
                    
                    });
                },
                TabPages::Parser => {
                    if self.current_token < 0 {
                        self.current_token = self.parser_tokens.len() as i32 - 1;
                    }

                    if self.current_token >= self.parser_tokens.len() as i32 {
                        self.current_token = 0;
                    }

                    ui.heading("Parser");

                    egui::Grid::new("LRColumns").show(ui, |ui| {

                        let token = &self.parser_tokens[self.current_token as usize];
                        let start = token.char_index;
                        let mut end = start + token.char_length;
                        if end > self.html_file.len() {
                            end = self.html_file.len();
                        }
                        
                        let p1 = &self.html_file[0..start];
                        let p2 = &self.html_file[start..end];
                        let p3 = &self.html_file[end..self.html_file.len()];

                        let style = egui::Style::default();
                        let mut layout_job = egui::text::LayoutJob::default();
                        egui::RichText::new(p1).append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                        egui::RichText::new(p2).background_color(egui::Color32::YELLOW).append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                        egui::RichText::new(p3).append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);

                        let label_response = ui.label(layout_job);
                        let label_height = label_response.rect.height();

                        let default_row_height = style.spacing.interact_size.y;

                        let rows = (label_height / default_row_height).floor() as usize;
                        let row_half = rows as i32 / 2;
                        
                        let mut beginning_index: i32 = self.current_token - row_half;
                        let ending_index = self.current_token + rows as i32 - row_half;
                        if beginning_index < 0 {
                            beginning_index = 0;
                        }
                        if ending_index >= self.parser_tokens.len() as i32{
                            beginning_index = (self.parser_tokens.len() - rows) as i32;
                        }

                        
                        egui::Grid::new("Grid")
                        .min_col_width(20.0)
                        .striped(true)
                        .show(ui, |ui| {
                            for i in beginning_index as usize..(beginning_index as usize + rows) {
                                let t = &self.parser_tokens[i];

                                let mut content_string = "None".to_string();
                                if t.content.is_some() {
                                    content_string = t.content.as_ref().unwrap().to_string();
                                }

                                if i == self.current_token as usize {
                                    ui.label(egui::RichText::new(format!("{:4}", t.position.1)).background_color(egui::Color32::YELLOW));
                                    ui.label(egui::RichText::new(format!("{:4}", t.position.0)).background_color(egui::Color32::YELLOW));
                                    ui.label(egui::RichText::new(format!("{:?}", t.tag_type)).background_color(egui::Color32::YELLOW));
                                    ui.label(egui::RichText::new(format!("{}", content_string)).background_color(egui::Color32::YELLOW));
                                }
                                else {
                                    ui.label(egui::RichText::new(format!("{:4}", t.position.1)));
                                    ui.label(egui::RichText::new(format!("{:4}", t.position.0)));
                                    ui.label(egui::RichText::new(format!("{:?}", t.tag_type)));
                                    ui.label(egui::RichText::new(format!("{}", content_string)));
                                }
                                
                                // ui.label(egui::RichText::new(format!("{:?}", t.position.1)).background_color(egui::Color32::RED));
                                // println!("{} {}", t.position.1, t.position.0);
                                ui.end_row();
                            }
                            
                        });
                        
                    
                    });
                },
                TabPages::Tree => {
                    if self.current_token < 0 {
                        self.current_token = self.tag_strings.len() as i32 - 1;
                    }

                    if self.current_token >= self.tag_strings.len() as i32 {
                        self.current_token = 0;
                    }
                    ui.heading("Tree");
                    egui::Grid::new("LRColumns").show(ui, |ui| {
                        let style = egui::Style::default();
                        let mut layout_job = egui::text::LayoutJob::default();
                        for i in 0..self.tag_strings.len() {
                            let tag_depth = self.tag_strings[i].0;
                            let string = &self.tag_strings[i].1;

                            for _ in 0..tag_depth {
                                egui::RichText::new("  ").append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                            }
                            egui::RichText::new("-").append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                            if self.current_token != i as i32 {
                                egui::RichText::new(string).append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                            }
                            else {
                                egui::RichText::new(string).background_color(egui::Color32::YELLOW).append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                            }
                            egui::RichText::new("\n").append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Center);
                        }

                        let label_response = ui.label(layout_job);
                        let label_height = label_response.rect.height();
                        let default_row_height = style.spacing.interact_size.y;

                        let rows = (label_height / default_row_height).floor() as usize;
                        let row_half = rows as i32 / 2;
                        
                        let mut beginning_index: i32 = self.current_token - row_half;
                        let ending_index = self.current_token + rows as i32 - row_half;
                        if beginning_index < 0 {
                            beginning_index = 0;
                        }
                        if ending_index >= self.tag_strings.len() as i32{
                            beginning_index = (self.tag_strings.len() - rows) as i32;
                        }

                        
                        egui::Grid::new("Grid")
                        .min_col_width(20.0)
                        .striped(true)
                        .show(ui, |ui| {
                            for i in beginning_index as usize..(beginning_index as usize + rows) {
                                let t = &self.tag_strings[i];

                                if i == self.current_token as usize {
                                    ui.label(egui::RichText::new(format!("{}", t.1)).background_color(egui::Color32::YELLOW));
                                }
                                else {
                                    ui.label(egui::RichText::new(format!("{}", t.1)));
                                }
                                
                                // ui.label(egui::RichText::new(format!("{:?}", t.position.1)).background_color(egui::Color32::RED));
                                // println!("{} {}", t.position.1, t.position.0);
                                ui.end_row();
                            }
                            
                        });
                    });
                }
            }
            

            

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
