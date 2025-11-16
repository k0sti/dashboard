use crate::agent::AgentId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    ToAgent,
    FromAgent,
    Broadcast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub is_toolcall: bool,
    pub is_error: bool,
    pub error_message: Option<String>,
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            is_toolcall: false,
            is_error: false,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: MessageId,
    pub agent_id: Option<AgentId>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub direction: MessageDirection,
    pub metadata: MessageMetadata,
}

impl ChatMessage {
    pub fn new_user_message(content: String, recipient: Option<AgentId>) -> Self {
        let direction = if recipient.is_some() {
            MessageDirection::ToAgent
        } else {
            MessageDirection::Broadcast
        };

        Self {
            id: MessageId::new(),
            agent_id: recipient,
            content,
            timestamp: Utc::now(),
            direction,
            metadata: MessageMetadata::default(),
        }
    }

    #[allow(dead_code)]
    pub fn new_agent_message(agent_id: AgentId, content: String) -> Self {
        Self {
            id: MessageId::new(),
            agent_id: Some(agent_id),
            content,
            timestamp: Utc::now(),
            direction: MessageDirection::FromAgent,
            metadata: MessageMetadata::default(),
        }
    }
}

pub fn render_chat_messages(
    ui: &mut egui::Ui,
    messages: &[ChatMessage],
    on_speak: &mut Option<MessageId>,
) {
    for message in messages {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let time_str = message.timestamp.format("%H:%M:%S").to_string();
                ui.label(
                    egui::RichText::new(&time_str)
                        .size(10.0)
                        .color(egui::Color32::GRAY),
                );

                match message.direction {
                    MessageDirection::FromAgent => {
                        if let Some(agent_id) = message.agent_id {
                            ui.label(
                                egui::RichText::new(format!("[Agent: {}]", agent_id))
                                    .strong()
                                    .color(egui::Color32::from_rgb(100, 150, 255)),
                            );
                        }
                    }
                    MessageDirection::ToAgent => {
                        ui.label(
                            egui::RichText::new("[You]")
                                .strong()
                                .color(egui::Color32::from_rgb(100, 200, 100)),
                        );
                        if let Some(agent_id) = message.agent_id {
                            ui.label(
                                egui::RichText::new(format!("→ {}", agent_id))
                                    .size(10.0)
                                    .color(egui::Color32::GRAY),
                            );
                        }
                    }
                    MessageDirection::Broadcast => {
                        ui.label(
                            egui::RichText::new("[You → All]")
                                .strong()
                                .color(egui::Color32::from_rgb(200, 150, 100)),
                        );
                    }
                }

                // Add TTS speak button
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🔊").on_hover_text("Speak this message").clicked() {
                        *on_speak = Some(message.id);
                    }
                });
            });

            if message.metadata.is_error {
                ui.colored_label(egui::Color32::RED, &message.content);
                if let Some(err_msg) = &message.metadata.error_message {
                    ui.colored_label(egui::Color32::DARK_RED, err_msg);
                }
            } else if message.metadata.is_toolcall {
                ui.colored_label(egui::Color32::from_rgb(200, 200, 100), &message.content);
            } else {
                ui.label(&message.content);
            }
        });
        ui.add_space(4.0);
    }
}
