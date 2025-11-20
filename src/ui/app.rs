use crate::agent::AgentId;
use crate::config::AppConfig;
use crate::plan::Plan;
use crate::storage::ChatHistoryStore;
use crate::tts::{TTSConfig, TTSService, TTSRequest};
use crate::ui::chat::{ChatMessage, MessageId};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{mpsc, Arc, Mutex};
use vte::{Params, Parser, Perform};

// Re-export TestMode type from main
pub type TestModeHandle = Arc<Mutex<crate::TestMode>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Home,
    Term,
}

// ANSI color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Default,
}

impl AnsiColor {
    fn to_egui_color(&self) -> egui::Color32 {
        match self {
            AnsiColor::Black => egui::Color32::from_rgb(0, 0, 0),
            AnsiColor::Red => egui::Color32::from_rgb(205, 49, 49),
            AnsiColor::Green => egui::Color32::from_rgb(13, 188, 121),
            AnsiColor::Yellow => egui::Color32::from_rgb(229, 229, 16),
            AnsiColor::Blue => egui::Color32::from_rgb(36, 114, 200),
            AnsiColor::Magenta => egui::Color32::from_rgb(188, 63, 188),
            AnsiColor::Cyan => egui::Color32::from_rgb(17, 168, 205),
            AnsiColor::White => egui::Color32::from_rgb(229, 229, 229),
            AnsiColor::BrightBlack => egui::Color32::from_rgb(102, 102, 102),
            AnsiColor::BrightRed => egui::Color32::from_rgb(241, 76, 76),
            AnsiColor::BrightGreen => egui::Color32::from_rgb(35, 209, 139),
            AnsiColor::BrightYellow => egui::Color32::from_rgb(245, 245, 67),
            AnsiColor::BrightBlue => egui::Color32::from_rgb(59, 142, 234),
            AnsiColor::BrightMagenta => egui::Color32::from_rgb(214, 112, 214),
            AnsiColor::BrightCyan => egui::Color32::from_rgb(41, 184, 219),
            AnsiColor::BrightWhite => egui::Color32::from_rgb(255, 255, 255),
            AnsiColor::Default => egui::Color32::from_rgb(229, 229, 229),
        }
    }
}

// Text styling attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStyle {
    pub fg_color: AnsiColor,
    pub bg_color: Option<AnsiColor>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            fg_color: AnsiColor::Default,
            bg_color: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

// Styled text segment
#[derive(Debug, Clone)]
pub struct StyledText {
    pub text: String,
    pub style: TextStyle,
}

// Output line can now contain styled text
#[derive(Debug, Clone)]
pub enum OutputLine {
    Styled(Vec<StyledText>),
    Stderr(String), // Keep stderr separate for debug messages
}

// Terminal performer that handles ANSI escape sequences
struct TerminalPerformer {
    output: Arc<Mutex<Vec<StyledText>>>,
    current_text: String,
    current_style: TextStyle,
    pending_cr: bool,  // Track if we just saw a \r without \n
}

impl TerminalPerformer {
    fn new(output: Arc<Mutex<Vec<StyledText>>>) -> Self {
        Self {
            output,
            current_text: String::new(),
            current_style: TextStyle::default(),
            pending_cr: false,
        }
    }

    fn flush_current_text(&mut self) {
        if !self.current_text.is_empty() {
            let mut output = self.output.lock().unwrap();
            output.push(StyledText {
                text: self.current_text.clone(),
                style: self.current_style,
            });
            self.current_text.clear();
        }
    }
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        // If we have a pending CR, clear the current line first
        if self.pending_cr {
            if let Some(last_newline) = self.current_text.rfind('\n') {
                self.current_text.truncate(last_newline + 1);
            } else {
                self.current_text.clear();
            }
            self.pending_cr = false;
        }
        self.current_text.push(c);
    }

    fn execute(&mut self, byte: u8) {
        // Handle control characters like \n, \r, \t
        match byte {
            b'\n' => {
                self.pending_cr = false;  // \n cancels pending CR
                self.current_text.push('\n');
            }
            b'\r' => {
                // Mark that we have a pending carriage return
                // The next print() will clear the current line
                self.pending_cr = true;
            }
            b'\t' => {
                self.current_text.push('\t');
            }
            b'\x08' => {
                // Backspace
                self.current_text.pop();
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        match action {
            'm' => {
                // SGR - Select Graphic Rendition (colors and styling)
                self.flush_current_text();

                let params: Vec<u16> = params.iter().map(|p| p[0]).collect();
                if params.is_empty() {
                    // Reset to default
                    self.current_style = TextStyle::default();
                } else {
                    let mut i = 0;
                    while i < params.len() {
                        match params[i] {
                            0 => self.current_style = TextStyle::default(),
                            1 => self.current_style.bold = true,
                            3 => self.current_style.italic = true,
                            4 => self.current_style.underline = true,
                            22 => self.current_style.bold = false,
                            23 => self.current_style.italic = false,
                            24 => self.current_style.underline = false,
                            // Foreground colors
                            30 => self.current_style.fg_color = AnsiColor::Black,
                            31 => self.current_style.fg_color = AnsiColor::Red,
                            32 => self.current_style.fg_color = AnsiColor::Green,
                            33 => self.current_style.fg_color = AnsiColor::Yellow,
                            34 => self.current_style.fg_color = AnsiColor::Blue,
                            35 => self.current_style.fg_color = AnsiColor::Magenta,
                            36 => self.current_style.fg_color = AnsiColor::Cyan,
                            37 => self.current_style.fg_color = AnsiColor::White,
                            39 => self.current_style.fg_color = AnsiColor::Default,
                            // Bright foreground colors
                            90 => self.current_style.fg_color = AnsiColor::BrightBlack,
                            91 => self.current_style.fg_color = AnsiColor::BrightRed,
                            92 => self.current_style.fg_color = AnsiColor::BrightGreen,
                            93 => self.current_style.fg_color = AnsiColor::BrightYellow,
                            94 => self.current_style.fg_color = AnsiColor::BrightBlue,
                            95 => self.current_style.fg_color = AnsiColor::BrightMagenta,
                            96 => self.current_style.fg_color = AnsiColor::BrightCyan,
                            97 => self.current_style.fg_color = AnsiColor::BrightWhite,
                            // Background colors
                            40 => self.current_style.bg_color = Some(AnsiColor::Black),
                            41 => self.current_style.bg_color = Some(AnsiColor::Red),
                            42 => self.current_style.bg_color = Some(AnsiColor::Green),
                            43 => self.current_style.bg_color = Some(AnsiColor::Yellow),
                            44 => self.current_style.bg_color = Some(AnsiColor::Blue),
                            45 => self.current_style.bg_color = Some(AnsiColor::Magenta),
                            46 => self.current_style.bg_color = Some(AnsiColor::Cyan),
                            47 => self.current_style.bg_color = Some(AnsiColor::White),
                            49 => self.current_style.bg_color = None,
                            // Bright background colors
                            100 => self.current_style.bg_color = Some(AnsiColor::BrightBlack),
                            101 => self.current_style.bg_color = Some(AnsiColor::BrightRed),
                            102 => self.current_style.bg_color = Some(AnsiColor::BrightGreen),
                            103 => self.current_style.bg_color = Some(AnsiColor::BrightYellow),
                            104 => self.current_style.bg_color = Some(AnsiColor::BrightBlue),
                            105 => self.current_style.bg_color = Some(AnsiColor::BrightMagenta),
                            106 => self.current_style.bg_color = Some(AnsiColor::BrightCyan),
                            107 => self.current_style.bg_color = Some(AnsiColor::BrightWhite),
                            _ => {}
                        }
                        i += 1;
                    }
                }
            }
            _ => {
                // Ignore other CSI sequences for now (cursor movement, clear screen, etc.)
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

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
    pub current_tab: AppTab,
    pub terminal_output: Vec<OutputLine>,
    pub terminal_input: String,
    pub terminal_startup_command: String,
    pub terminal_stdin_tx: Option<mpsc::Sender<String>>,
    pub terminal_stdout_rx: Option<mpsc::Receiver<OutputLine>>,
    pub terminal_pty_master: Option<Box<dyn MasterPty + Send>>,
    pub terminal_pty_size: PtySize,
    pub test_mode: Option<TestModeHandle>,
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
            current_tab: AppTab::Term,
            terminal_output: Vec::new(),
            terminal_input: String::new(),
            terminal_startup_command: String::from("bash"),
            terminal_stdin_tx: None,
            terminal_stdout_rx: None,
            terminal_pty_master: None,
            terminal_pty_size: PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            },
            test_mode: None,
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

    pub fn spawn_terminal(&mut self) {
        if self.terminal_stdin_tx.is_some() {
            return; // Already spawned
        }

        let command = self.terminal_startup_command.clone();
        let pty_size = self.terminal_pty_size;
        let (stdin_tx, stdin_rx) = mpsc::channel::<String>();
        let (output_tx, output_rx) = mpsc::channel::<OutputLine>();

        std::thread::spawn(move || {
            // Initialize PTY system
            let pty_system = native_pty_system();

            // Create PTY pair
            let pair = match pty_system.openpty(pty_size) {
                Ok(pair) => pair,
                Err(e) => {
                    let _ = output_tx.send(OutputLine::Stderr(format!("Failed to create PTY: {}\n", e)));
                    return;
                }
            };

            let master = pair.master;
            let slave = pair.slave;

            // Parse command using shell-words for proper argument handling
            let mut cmd = CommandBuilder::new("sh");
            cmd.arg("-c");
            cmd.arg(&command);

            // Spawn child process with PTY
            let mut child = match slave.spawn_command(cmd) {
                Ok(child) => child,
                Err(e) => {
                    let _ = output_tx.send(OutputLine::Stderr(format!("Failed to spawn process: {}\n", e)));
                    let _ = output_tx.send(OutputLine::Stderr(format!("Command was: sh -c '{}'\n", command)));
                    return;
                }
            };

            // Clone master for reader thread
            let reader = match master.try_clone_reader() {
                Ok(reader) => reader,
                Err(e) => {
                    let _ = output_tx.send(OutputLine::Stderr(format!("Failed to clone PTY reader: {}\n", e)));
                    return;
                }
            };

            // Spawn PTY reader thread with ANSI parser
            let output_tx_clone = output_tx.clone();
            std::thread::spawn(move || {
                use std::time::{Duration, Instant};

                let mut reader = reader;
                let mut buffer = [0u8; 1024];

                // Create VTE parser and performer
                let styled_segments = Arc::new(Mutex::new(Vec::new()));
                let mut performer = TerminalPerformer::new(styled_segments.clone());
                let mut parser = Parser::new();
                let mut last_output_time = Instant::now();
                let flush_delay = Duration::from_millis(10);  // Small delay to batch rapid updates

                loop {
                    // Try to read with a small timeout
                    match reader.read(&mut buffer) {
                        Ok(0) => {
                            // EOF - flush any remaining text and exit
                            performer.flush_current_text();
                            let segments = styled_segments.lock().unwrap().clone();
                            if !segments.is_empty() {
                                let _ = output_tx_clone.send(OutputLine::Styled(segments));
                            }
                            break;
                        }
                        Ok(n) => {
                            // Parse bytes through VTE parser
                            for byte in &buffer[..n] {
                                parser.advance(&mut performer, *byte);
                            }

                            // Only send if enough time has passed OR if we hit a newline
                            let now = Instant::now();
                            let should_flush = now.duration_since(last_output_time) >= flush_delay;

                            if should_flush {
                                performer.flush_current_text();
                                let mut segments_guard = styled_segments.lock().unwrap();
                                if !segments_guard.is_empty() {
                                    if output_tx_clone.send(OutputLine::Styled(segments_guard.clone())).is_err() {
                                        break;
                                    }
                                    segments_guard.clear();
                                    last_output_time = now;
                                }
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            });

            // Get writer from master PTY
            let mut writer = master.take_writer().expect("Failed to get PTY writer");

            // PTY writer loop (runs in spawning thread)
            while let Ok(input) = stdin_rx.recv() {
                if writer.write_all(input.as_bytes()).is_err() {
                    break;
                }
                if writer.flush().is_err() {
                    break;
                }
            }

            // Wait for child process
            let _ = child.wait();
        });

        self.terminal_stdin_tx = Some(stdin_tx);
        self.terminal_stdout_rx = Some(output_rx);
    }

    pub fn reset_terminal(&mut self) {
        // Drop existing channels and PTY
        self.terminal_stdin_tx = None;
        self.terminal_stdout_rx = None;
        self.terminal_pty_master = None;
        self.terminal_output.clear();

        // Spawn new terminal with updated command
        self.spawn_terminal();
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check test mode exit conditions
        if let Some(ref test_mode) = self.test_mode {
            if test_mode.lock().unwrap().should_exit() {
                std::process::exit(0);
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Agent Dashboard");
                ui.separator();

                // Tab selector
                ui.selectable_value(&mut self.current_tab, AppTab::Home, "Home");
                ui.selectable_value(&mut self.current_tab, AppTab::Term, "Term");

                ui.separator();

                // Show buttons only on Home tab
                if self.current_tab == AppTab::Home {
                    if ui.button("Config").clicked() {
                        self.show_config_panel = !self.show_config_panel;
                    }

                    if ui.button("Plans").clicked() {
                        self.show_plan_panel = !self.show_plan_panel;
                    }

                    if ui.button("TTS").clicked() {
                        self.show_tts_panel = !self.show_tts_panel;
                    }
                }
            });
        });

        // Show content based on current tab
        match self.current_tab {
            AppTab::Home => self.render_home_tab(ctx),
            AppTab::Term => self.render_term_tab(ctx),
        }
    }
}

impl DashboardApp {
    fn render_home_tab(&mut self, ctx: &egui::Context) {
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

    fn render_term_tab(&mut self, ctx: &egui::Context) {
        // Spawn terminal process if not already running
        if self.terminal_stdin_tx.is_none() {
            self.spawn_terminal();
        }

        // Poll for output updates
        if let Some(ref mut output_rx) = self.terminal_stdout_rx {
            while let Ok(line) = output_rx.try_recv() {
                // Log to test mode if enabled
                if let Some(ref test_mode) = self.test_mode {
                    match &line {
                        OutputLine::Styled(segments) => {
                            let text: String = segments.iter().map(|s| s.text.as_str()).collect();
                            test_mode.lock().unwrap().log(text);
                        }
                        OutputLine::Stderr(text) => {
                            test_mode.lock().unwrap().log(text.clone());
                        }
                    }
                }

                self.terminal_output.push(line);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Startup command field at the top
                ui.horizontal(|ui| {
                    ui.label("Startup Command:");
                    let cmd_response = ui.add(
                        egui::TextEdit::singleline(&mut self.terminal_startup_command)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace)
                    );

                    // Reset terminal when Enter is pressed on startup command
                    if cmd_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.reset_terminal();
                        cmd_response.request_focus();
                    }
                });

                ui.separator();

                // Calculate available height for output and input
                let spacing = ui.spacing().item_spacing.y;
                let available_height = ui.available_height();
                let input_height = 80.0; // Height for input area
                let separator_height = spacing * 2.0;
                let output_height = available_height - input_height - separator_height;

                // Terminal output area - fills remaining vertical space
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .max_height(output_height)
                    .show(ui, |ui| {
                        ui.style_mut().override_font_id = Some(egui::FontId::monospace(14.0));

                        // Build a LayoutJob to combine all styled segments without extra newlines
                        let mut layout_job = egui::text::LayoutJob::default();

                        for output_line in &self.terminal_output {
                            match output_line {
                                OutputLine::Styled(segments) => {
                                    for segment in segments {
                                        let fg_color = segment.style.fg_color.to_egui_color();

                                        let format = egui::TextFormat {
                                            font_id: egui::FontId::monospace(14.0),
                                            color: fg_color,
                                            ..Default::default()
                                        };

                                        layout_job.append(&segment.text, 0.0, format);
                                    }
                                }
                                OutputLine::Stderr(text) => {
                                    let format = egui::TextFormat {
                                        font_id: egui::FontId::monospace(14.0),
                                        color: egui::Color32::from_rgb(255, 80, 80),
                                        ..Default::default()
                                    };
                                    layout_job.append(text, 0.0, format);
                                }
                            }
                        }

                        // Render as a single label with styled text
                        ui.label(layout_job);
                    });

                ui.separator();

                // Terminal input at the bottom - multiline with Enter to send
                ui.vertical(|ui| {
                    ui.label("Input (Shift+Enter for newline, Enter to send):");

                    let text_edit = egui::TextEdit::multiline(&mut self.terminal_input)
                        .desired_width(f32::INFINITY)
                        .desired_rows(2)
                        .font(egui::TextStyle::Monospace);

                    let response = ui.add(text_edit);

                    // Check if Enter was pressed without Shift
                    if response.has_focus() {
                        let enter_pressed = ui.input(|i| {
                            i.key_pressed(egui::Key::Enter) && !i.modifiers.shift
                        });

                        if enter_pressed {
                            // Send command to terminal stdin
                            if !self.terminal_input.trim().is_empty() {
                                if let Some(ref stdin_tx) = self.terminal_stdin_tx {
                                    let command = format!("{}\n", self.terminal_input);
                                    let _ = stdin_tx.send(command);
                                }
                            } else {
                                // Send just newline for empty input
                                if let Some(ref stdin_tx) = self.terminal_stdin_tx {
                                    let _ = stdin_tx.send("\n".to_string());
                                }
                            }
                            self.terminal_input.clear();
                            response.request_focus();
                        }
                    }
                });
            });
        });

        // Request continuous repaint to update terminal output
        ctx.request_repaint();
    }
}
