// src/app.rs

use crate::actions::command::{self, Command};
use crate::game::ai::{AiAction, AiMind};
use crate::game::node::StatusEffect;
use crate::game::state::GameState;
use crate::ui::{self, tui::Tui};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::time::Duration;

pub struct App {
    pub exit: bool,
    pub game_state: GameState,
    pub input_text: String,
    pub system_log: Vec<Line<'static>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            game_state: GameState::new(),
            input_text: String::new(),
            system_log: vec![Line::from(
                "Welcome to Integrity Zero. Type 'help()' for commands.",
            )],
        }
    }

    pub fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        ui::renderer::render_main_ui(frame, self);
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        let command_str = self.input_text.drain(..).collect::<String>();
                        if !command_str.is_empty() {
                            let player_input_line = Line::from(vec![
                                Span::styled("> ", Style::default().fg(Color::Cyan)),
                                Span::from(command_str.clone()),
                            ]);
                            self.system_log.push(player_input_line);

                            let command = command::parse_command(&command_str);
                            self.dispatch(command);
                        }
                    }
                    KeyCode::Char(c) => {
                        self.input_text.push(c);
                    }
                    KeyCode::Backspace => {
                        self.input_text.pop();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn dispatch(&mut self, command: Command) {
        match command {
            Command::Help(arg) => self.handle_help(arg),
            Command::Invalid => {
                self.system_log.push(Line::from(Span::styled(
                    "Error: Invalid command. Use help() for details.",
                    Style::default().fg(Color::Red),
                )));
            }
            _ => {
                if self.handle_player_action(command) {
                    self.run_ai_turn();
                }
            }
        }
    }
}

// Implementazioni per la logica di gioco
impl App {
    fn handle_player_action(&mut self, command: Command) -> bool {
        match command {
            Command::Fortify(node_name) => self.handle_fortify(node_name),
            Command::Scan(node_name) => self.handle_scan(node_name),
            Command::Quit => {
                self.exit = true;
                false
            }
            Command::Help(_) | Command::Invalid => false,
        }
    }

    fn run_ai_turn(&mut self) {
        self.system_log.push(Line::from(Span::styled(
            "--- Black Hat Turn ---",
            Style::default().fg(Color::Magenta),
        )));
        if let Some(ai_action) = AiMind::decide_action(&self.game_state.nodes) {
            self.apply_ai_action(ai_action);
        } else {
            self.system_log.push(Line::from(Span::styled(
                "[INFO] AI has no available targets.",
                Style::default().fg(Color::DarkGray),
            )));
        }
        self.system_log.push(Line::from(Span::styled(
            "--- White Hat Turn ---",
            Style::default().fg(Color::Cyan),
        )));
    }

    fn apply_ai_action(&mut self, action: AiAction) {
        match action {
            AiAction::Exploit(target_id) => {
                if let Some(node) = self.game_state.nodes.iter_mut().find(|n| n.id == target_id)
                {
                    node.hp = node.hp.saturating_sub(15);
                    self.system_log.push(Line::from(vec![
                        Span::styled("[ATTACK] ", Style::default().fg(Color::Red)),
                        Span::from(format!(
                            "Black Hat launched an Exploit against {}. HP is now {}.",
                            node.name, node.hp
                        )),
                    ]));
                }
            }
            AiAction::Weaken(target_id) => {
                if let Some(node) = self.game_state.nodes.iter_mut().find(|n| n.id == target_id)
                {
                    if !node.status_effects.contains(&StatusEffect::Vulnerability) {
                        node.status_effects.push(StatusEffect::Vulnerability);
                        self.system_log.push(Line::from(vec![
                            Span::styled("[ATTACK] ", Style::default().fg(Color::Red)),
                            Span::from(format!(
                                "Black Hat weakened {}. It is now vulnerable.",
                                node.name
                            )),
                        ]));
                    }
                }
            }
        }
    }

    // *** MODIFICA CHIAVE QUI ***
    // La closure ora accetta un `&mut Vec<Line<'static>>`.
    fn with_mutable_node<F>(&mut self, node_name: &str, action: F) -> bool
    where
        F: FnOnce(&mut crate::game::node::Node, &mut Vec<Line<'static>>),
    {
        if let Some(node) = self
            .game_state
            .nodes
            .iter_mut()
            .find(|n| n.name.eq_ignore_ascii_case(node_name))
        {
            action(node, &mut self.system_log);
            true
        } else {
            self.system_log.push(Line::from(Span::styled(
                format!("Error: Node '{}' not found.", node_name),
                Style::default().fg(Color::Red),
            )));
            false
        }
    }
}

// Implementazioni per i comandi specifici del giocatore
impl App {
    fn handle_help(&mut self, arg: Option<String>) {
        let help_text = match arg {
            Some(topic) => {
                let topic_lower = topic.to_lowercase();
                if topic_lower == "fortify" {
                    "--- COMMAND: Fortify ---\nSyntax: Fortify(NodeName)\nAP Cost: 2\nEffect: Restores 25 HP to a node and applies [Shield] status, which absorbs the next attack.".to_string()
                } else if topic_lower == "scan" {
                    "--- COMMAND: Scan ---\nSyntax: Scan(NodeName)\nAP Cost: 1\nEffect: Reveals hidden threats like [Backdoor] on a node.".to_string()
                } else if let Some(node) = self.game_state.nodes.iter().find(|n| n.name.to_lowercase() == topic_lower) {
                    match node.node_type {
                        crate::game::node::NodeType::Database => "--- NODE: Database [DB] ---\nCore of the system. Stores critical data. If destroyed, you lose the game. Vulnerable to [SQL_Injection].".to_string(),
                        crate::game::node::NodeType::Firewall => "--- NODE: Firewall [FW] ---\nDefensive node. Takes 25% less damage from all attacks.".to_string(),
                        crate::game::node::NodeType::Authentication => "--- NODE: Authentication [AUTH] ---\nManages logins and permissions. If compromised, the Black Hat gains extra actions.".to_string(),
                        _ => format!("--- NODE: {} ---\nNo special properties known.", node.name),
                    }
                } else {
                    format!("Help topic '{}' not found.", topic)
                }
            }
            None => {
                "--- AVAILABLE COMMANDS ---\n  Fortify(NodeName)\n  Scan(NodeName)\n  help(topic)\n  quit\n\nFor details, use help(command_name) or help(NodeName).".to_string()
            }
        };
        self.system_log
            .extend(help_text.split('\n').map(|s| {
                Line::from(Span::styled(
                    s.to_string(),
                    Style::default().fg(Color::DarkGray),
                ))
            }));
    }

    fn handle_fortify(&mut self, node_name: String) -> bool {
        const AP_COST: u8 = 2;
        if self.game_state.player_ap < AP_COST {
            self.system_log.push(Line::from(Span::styled(
                "Error: Not enough Action Points.",
                Style::default().fg(Color::Yellow),
            )));
            return false;
        }

        let success = self.with_mutable_node(&node_name, |node, log| {
            node.hp = node.hp.saturating_add(25);
            node.hp = std::cmp::min(node.hp, node.max_hp);

            if !node.status_effects.contains(&StatusEffect::Shield) {
                node.status_effects.push(StatusEffect::Shield);
            }
            log.push(Line::from(vec![
                Span::styled("[SUCCESS] ", Style::default().fg(Color::Green)),
                Span::from(format!(
                    "Fortified node {}. HP is now {}.",
                    node.name, node.hp
                )),
            ]));
        });

        if success {
            self.game_state.player_ap -= AP_COST;
        }
        success
    }

    fn handle_scan(&mut self, node_name: String) -> bool {
        const AP_COST: u8 = 1;
        if self.game_state.player_ap < AP_COST {
            self.system_log.push(Line::from(Span::styled(
                "Error: Not enough Action Points.",
                Style::default().fg(Color::Yellow),
            )));
            return false;
        }

        let success = self.with_mutable_node(&node_name, |node, log| {
            log.push(Line::from(vec![
                Span::styled("[INFO] ", Style::default().fg(Color::Blue)),
                Span::from(format!(
                    "Scan of node {} complete. No hidden threats detected.",
                    node.name
                )),
            ]));
        });

        if success {
            self.game_state.player_ap -= AP_COST;
        }
        success
    }
}