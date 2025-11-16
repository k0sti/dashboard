use crate::agent::AgentId;
use crate::config::AppConfig;
use crate::plan::Plan;
use crate::storage::ChatHistoryStore;
use crate::tts::{TTSConfig, TTSService, TTSRequest};
use crate::ui::chat::{ChatMessage, MessageId};
use std::collections::HashMap;

pub struct DashboardApp {
    pub config: AppConfig,
    pub active_agents: HashMap<AgentId, String>,
    pub selected_agent: Option<AgentId>,
    pub broadcast_mode: bool,
    pub chat_messages: Vec<ChatMessage>,
    pub chat_input: String,
    pub show_config_panel: bool,
    pub show_plan_panel: bool,
    pub show_tts_panel: bool,
    pub plans: Vec<Plan>,
    #[allow(dead_code)]
    pub chat_history_store: Option<ChatHistoryStore>,
    pub tts_config: TTSConfig,
    pub tts_service: Option<TTSService>,
    pub speak_message_id: Option<MessageId>,
}

impl DashboardApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let chat_history_store = ChatHistoryStore::new().ok();

        // Initialize TTS from saved config
        let tts_config = config.tts.clone();
        let tts_service = if tts_config.enabled {
            TTSService::start(tts_config.clone()).ok()
        } else {
            None
        };

        Self {
            config,
            active_agents: HashMap::new(),
            selected_agent: None,
            broadcast_mode: false,
            chat_messages: Vec::new(),
            chat_input: String::new(),
            show_config_panel: false,
            show_plan_panel: false,
            show_tts_panel: false,
            plans: Vec::new(),
            chat_history_store,
            tts_config,
            tts_service,
            speak_message_id: None,
        }
    }

    pub fn save_tts_config(&mut self) {
        self.config.tts = self.tts_config.clone();
        if let Err(e) = self.config.save() {
            log::error!("Failed to save TTS config: {}", e);
        }
    }

    pub fn send_message(&mut self) {
        if self.chat_input.trim().is_empty() {
            return;
        }

        let content = self.chat_input.clone();
        let recipient = if self.broadcast_mode {
            None
        } else {
            self.selected_agent
        };

        let message = ChatMessage::new_user_message(content, recipient);
        self.chat_messages.push(message);

        self.chat_input.clear();
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Agent Dashboard");
                ui.separator();

                if ui.button("Config").clicked() {
                    self.show_config_panel = !self.show_config_panel;
                }

                if ui.button("Plans").clicked() {
                    self.show_plan_panel = !self.show_plan_panel;
                }

                if ui.button("TTS").clicked() {
                    self.show_tts_panel = !self.show_tts_panel;
                }
            });
        });

        egui::SidePanel::left("agents_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Active Agents");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.active_agents.is_empty() {
                        ui.label("No active agents");
                    } else {
                        for (agent_id, agent_name) in &self.active_agents {
                            let is_selected = self.selected_agent.as_ref() == Some(agent_id);
                            if ui
                                .selectable_label(is_selected, agent_name)
                                .clicked()
                            {
                                self.selected_agent = Some(*agent_id);
                                self.broadcast_mode = false;
                            }
                        }
                    }

                    ui.separator();

                    if ui
                        .selectable_label(self.broadcast_mode, "📢 Broadcast")
                        .clicked()
                    {
                        self.broadcast_mode = true;
                        self.selected_agent = None;
                    }
                });
            });

        if self.show_config_panel {
            super::config_panel::show_config_panel(ctx, self);
        }

        if self.show_plan_panel {
            egui::SidePanel::right("plans_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Plans");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.plans.is_empty() {
                            ui.label("No active plans");
                        } else {
                            for plan in &self.plans {
                                ui.group(|ui| {
                                    ui.label(&plan.title);
                                    ui.label(format!("Steps: {}", plan.steps.len()));
                                });
                            }
                        }
                    });
                });
        }

        if self.show_tts_panel {
            egui::SidePanel::right("tts_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Text-to-Speech");
                    ui.separator();

                    ui.checkbox(&mut self.tts_config.enabled, "Enable TTS");

                    if self.tts_config.enabled {
                        ui.checkbox(&mut self.tts_config.auto_speak, "Auto-speak agent messages");

                        ui.separator();

                        ui.label("Playback Speed:");
                        ui.add(egui::Slider::new(&mut self.tts_config.playback_speed, 0.5..=2.0));

                        ui.separator();

                        ui.label("Voice Model:");
                        ui.text_edit_singleline(&mut self.tts_config.selected_voice);

                        ui.separator();

                        if ui.button("Apply Settings").clicked() {
                            self.tts_config.validate();
                            // Save config
                            self.save_tts_config();
                            // Restart TTS service with new config
                            if let Ok(service) = TTSService::start(self.tts_config.clone()) {
                                self.tts_service = Some(service);
                            }
                        }

                        if let Some(ref service) = self.tts_service {
                            ui.separator();
                            ui.label("TTS Service: Running");

                            if ui.button("Stop Playback").clicked() {
                                let service = service.clone();
                                tokio::spawn(async move {
                                    let _ = service.stop().await;
                                });
                            }

                            if ui.button("Clear Queue").clicked() {
                                let service = service.clone();
                                tokio::spawn(async move {
                                    let _ = service.clear_queue().await;
                                });
                            }
                        } else {
                            ui.separator();
                            ui.label("TTS Service: Stopped");
                        }
                    } else {
                        if let Some(ref service) = self.tts_service {
                            let service_clone = service.clone();
                            tokio::spawn(async move {
                                let _ = service_clone.shutdown().await;
                            });
                            self.tts_service = None;
                        }
                    }

                    ui.separator();
                    ui.label("Model Directory:");
                    ui.label(self.tts_config.model_directory.display().to_string());

                    ui.separator();
                    ui.label("Note: Place Piper voice models (.onnx + .json) in the model directory.");
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let available_height = ui.available_height();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .max_height(available_height - 80.0)
                    .show(ui, |ui| {
                        super::chat::render_chat_messages(ui, &self.chat_messages, &mut self.speak_message_id);
                    });

                // Handle speak requests
                if let Some(msg_id) = self.speak_message_id.take() {
                    if let Some(message) = self.chat_messages.iter().find(|m| m.id == msg_id) {
                        if let Some(ref service) = self.tts_service {
                            let request = TTSRequest::new(
                                message.content.clone(),
                                self.tts_config.selected_voice.clone(),
                                self.tts_config.playback_speed,
                            );
                            let service = service.clone();
                            tokio::spawn(async move {
                                if let Err(e) = service.speak(request).await {
                                    log::error!("TTS speak error: {}", e);
                                }
                            });
                        }
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("To:");
                    if self.broadcast_mode {
                        ui.label("All agents");
                    } else if let Some(agent_id) = self.selected_agent {
                        if let Some(name) = self.active_agents.get(&agent_id) {
                            ui.label(name);
                        } else {
                            ui.label("None");
                        }
                    } else {
                        ui.label("None (select an agent)");
                    }
                });

                ui.horizontal(|ui| {
                    let text_edit = egui::TextEdit::multiline(&mut self.chat_input)
                        .desired_width(f32::INFINITY)
                        .desired_rows(2);

                    ui.add(text_edit);

                    if ui.button("Send").clicked()
                        || (ui.input(|i| {
                            i.key_pressed(egui::Key::Enter)
                                && !i.modifiers.shift
                        }))
                    {
                        self.send_message();
                    }
                });
            });
        });
    }
}
