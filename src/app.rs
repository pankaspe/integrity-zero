// src/app.rs

use crate::actions::command::{self, Command};
use crate::game::ai::{AiAction, AiMind};
use crate::game::node::StatusEffect;
use crate::game::state::GameState;
use crate::ui::{self, tui::Tui};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::Frame;
use std::time::Duration;

pub struct App {
    pub exit: bool,
    pub game_state: GameState,
    pub input_text: String,
    pub system_log: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            game_state: GameState::new(),
            input_text: String::new(),
            system_log: vec![
                "Welcome to Integrity Zero. Type 'help()' for a list of commands.".to_string(),
            ],
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
                        // Se l'utente preme invio a vuoto, non fare nulla
                        if !command_str.is_empty() {
                            self.system_log.push(format!("> {}", command_str));
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
                    // Il caso per 'q' è stato rimosso.
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
                self.system_log.push("Error: Invalid command. Use help() for details.".to_string());
            }
            // Tutti gli altri comandi vengono gestiti qui
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
                false // L'azione di quit non consuma un turno
            }
            // Non serve il caso `_` perché `Help` e `Invalid` sono già gestiti in `dispatch`
            // e questo `match` gestisce tutti gli altri casi dell'enum `Command`.
            Command::Help(_) => false, // Non dovrebbe accadere, ma non consuma turno
            Command::Invalid => false, // Non dovrebbe accadere, ma non consuma turno
        }
    }

    fn run_ai_turn(&mut self) {
        self.system_log.push("--- Black Hat Turn ---".to_string());
        if let Some(ai_action) = AiMind::decide_action(&self.game_state.nodes) {
            self.apply_ai_action(ai_action);
        } else {
            self.system_log.push("[INFO] AI has no available targets.".to_string());
        }
        self.system_log.push("--- White Hat Turn ---".to_string());
    }

    fn apply_ai_action(&mut self, action: AiAction) {
        match action {
            AiAction::Exploit(target_id) => {
                if let Some(node) = self.game_state.nodes.iter_mut().find(|n| n.id == target_id) {
                    node.hp = node.hp.saturating_sub(15);
                    self.system_log.push(format!(
                        "[ATTACK] Black Hat launched an Exploit against {}. HP is now {}.",
                        node.name, node.hp
                    ));
                }
            }
            AiAction::Weaken(target_id) => {
                if let Some(node) = self.game_state.nodes.iter_mut().find(|n| n.id == target_id) {
                    if !node.status_effects.contains(&StatusEffect::Vulnerability) {
                        node.status_effects.push(StatusEffect::Vulnerability);
                        self.system_log.push(format!(
                            "[ATTACK] Black Hat weakened {}. It is now [V]ulnerable.",
                            node.name
                        ));
                    }
                }
            }
        }
    }

    fn with_mutable_node<F>(&mut self, node_name: &str, action: F) -> bool
    where
        F: FnOnce(&mut crate::game::node::Node, &mut Vec<String>),
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
            self.system_log.push(format!("Error: Node '{}' not found.", node_name));
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
                    "--- COMANDO: Fortify ---\nSintassi: Fortify(NomeNodo)\nCosto AP: 2\nEffetto: Ripristina 25 HP di un nodo e applica [S]hield.".to_string()
                } else if topic_lower == "scan" {
                    "--- COMANDO: Scan ---\nSintassi: Scan(NomeNodo)\nCosto AP: 1\nEffetto: Rivela minacce nascoste su un nodo.".to_string()
                } else if let Some(node) = self.game_state.nodes.iter().find(|n| n.name.to_lowercase() == topic_lower) {
                    match node.node_type {
                        crate::game::node::NodeType::Database => "--- NODO: Database [DB] ---\nContiene i dati critici. Se distrutto, la partita è persa. È vulnerabile a [SQL_Injection].".to_string(),
                        crate::game::node::NodeType::Firewall => "--- NODO: Firewall [FW] ---\nNodo difensivo. Subisce il 25% di danni in meno da tutti gli attacchi.".to_string(),
                        _ => format!("--- NODO: {} ---\nNessuna proprietà speciale nota.", node.name),
                    }
                } else {
                    format!("Help topic '{}' not found.", topic)
                }
            }
            None => {
                "--- COMANDI DISPONIBILI ---\n  Fortify(NomeNodo)\n  Scan(NomeNodo)\n  help(topic)\n  quit\nPer info, usa help(comando) o help(NomeNodo).".to_string()
            }
        };
        self.system_log.extend(help_text.split('\n').map(|s| s.to_string()));
    }

    fn handle_fortify(&mut self, node_name: String) -> bool {
        const AP_COST: u8 = 2;
        if self.game_state.player_ap < AP_COST {
            self.system_log.push("Error: Not enough Action Points.".to_string());
            return false;
        }

        let success = self.with_mutable_node(&node_name, |node, log| {
            node.hp = node.hp.saturating_add(25);
            node.hp = std::cmp::min(node.hp, node.max_hp);

            if !node.status_effects.contains(&StatusEffect::Shield) {
                node.status_effects.push(StatusEffect::Shield);
            }
            log.push(format!("[SUCCESS] Fortified node {}. HP is now {}.", node.name, node.hp));
        });

        if success {
            self.game_state.player_ap -= AP_COST;
        }
        success
    }

    fn handle_scan(&mut self, node_name: String) -> bool {
        const AP_COST: u8 = 1;
        if self.game_state.player_ap < AP_COST {
            self.system_log.push("Error: Not enough Action Points.".to_string());
            return false;
        }

        let success = self.with_mutable_node(&node_name, |node, log| {
            log.push(format!("[INFO] Scan of node {} complete. No hidden threats detected.", node.name));
        });
        
        if success {
            self.game_state.player_ap -= AP_COST;
        }
        success
    }
}