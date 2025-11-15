use crate::agent::{AgentConfig, AgentId, AgentType};
use crate::ui::app::DashboardApp;

pub fn show_config_panel(ctx: &egui::Context, app: &mut DashboardApp) {
    egui::Window::new("Agent Configuration")
        .open(&mut app.show_config_panel)
        .default_width(500.0)
        .show(ctx, |ui| {
            ui.heading("Configured Agents");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Add Agent").clicked() {
                    // Create a default Ollama agent config
                    let config = AgentConfig {
                        id: AgentId::new(),
                        name: format!("Agent {}", app.config.agents.len() + 1),
                        agent_type: AgentType::Ollama,
                        config_data: serde_json::json!({
                            "host": "http://localhost:11434",
                            "model": "llama2"
                        }),
                    };
                    app.config.add_agent(config);
                    let _ = app.config.save();
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut to_remove: Option<AgentId> = None;

                for agent in &app.config.agents {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&agent.name).strong());
                            ui.label(format!("({})", agent.agent_type));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Delete").clicked() {
                                    to_remove = Some(agent.id);
                                }

                                if ui.button("Connect").clicked() {
                                    app.active_agents
                                        .insert(agent.id, agent.name.clone());
                                }
                            });
                        });

                        ui.label(format!(
                            "Config: {}",
                            serde_json::to_string_pretty(&agent.config_data)
                                .unwrap_or_default()
                        ));
                    });

                    ui.add_space(8.0);
                }

                if let Some(id) = to_remove {
                    app.config.remove_agent(&id);
                    app.active_agents.remove(&id);
                    let _ = app.config.save();
                }
            });
        });
}
