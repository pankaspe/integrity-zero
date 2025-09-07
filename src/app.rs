// src/app.rs

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::Frame;
use std::time::Duration;

use crate::ui::{self, tui::Tui}; 
use crate::actions::command::{self, Command};
use crate::game::state::GameState;

// La struct che conterrà tutto lo stato della nostra applicazione.
pub struct App {
    pub exit: bool,
    pub game_state: GameState,
    pub input_text: String,
    pub system_log: Vec<String>,
}

impl App {
    // La funzione "costruttore" per creare una nuova istanza di App.
    pub fn new() -> Self {
        Self { 
            game_state: GameState::new(),
            input_text: String::new(),
            system_log: vec!["Welcome to Integrity Zero. Type 'help()' for a list of commands.".to_string()],
            exit: false
        }
    }

    // Il loop principale del gioco.
    pub fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        // Continua a girare finché `self.exit` non è `true`.
        while !self.exit {
            // Disegna l'interfaccia.
            terminal.draw(|frame| self.render_frame(frame))?;
            // Gestisce gli eventi (input utente). Se restituisce un errore, il programma termina.
            self.handle_events()?;
        }
        Ok(())
    }

    // Un metodo dedicato al rendering di un singolo frame.
    fn render_frame(&self, frame: &mut Frame) {
        // Passiamo lo stato dell'app (`self`) al nostro renderer.
        ui::renderer::render_main_ui(frame, self);
    }

    // Dispatch si occupa di "smistare" il comando alla logica corretta.
    fn dispatch(&mut self, command: Command) {
        match command {
            Command::Help(arg) => self.handle_help(arg), // Chiamiamo una funzione helper
            Command::Fortify(node_name) => self.handle_fortify(node_name),
            Command::Scan(node_name) => self.handle_scan(node_name),
            Command::Quit => {
                self.exit = true;
            }
            Command::Invalid => {
                self.system_log.push("Error: Invalid command. Use help() for details.".to_string());
            }
        }
    }

    // Un metodo dedicato alla gestione degli eventi.
    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    // `match` è un'espressione, possiamo usarla per assegnare un valore.
                    // Qui non lo facciamo, ma è utile saperlo.
                    match key.code {
                        KeyCode::Char('q') => self.exit = true,
                        // Quando l'utente preme Invio
                        KeyCode::Enter => {
                            let command_str = self.input_text.drain(..).collect::<String>();
                            self.system_log.push(format!("> {}", command_str));

                            // Usiamo il nostro parser
                            let command = command::parse_command(&command_str);

                            // Applichiamo il comando allo stato del gioco
                            self.dispatch(command);
                        }
                        // Quando l'utente scrive un carattere
                        KeyCode::Char(c) => {
                            self.input_text.push(c);
                        }
                        // Quando l'utente preme backspace
                        KeyCode::Backspace => {
                            self.input_text.pop();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl App {
    fn handle_help(&mut self, arg: Option<String>) {
        let help_text = match arg {
            // Se c'è un argomento, es. `help(Database)`
            Some(topic) => {
                // Controlla se l'argomento è un nome di comando
                if topic.to_lowercase() == "fortify" {
                    "--- COMANDO: Fortify ---\nSintassi: Fortify(NomeNodo)\nCosto AP: 2\nEffetto: Ripristina 25 HP di un nodo e applica [S]hield.".to_string()
                } else if topic.to_lowercase() == "scan" {
                     "--- COMANDO: Scan ---\nSintassi: Scan(NomeNodo)\nCosto AP: 1\nEffetto: Rivela minacce nascoste su un nodo.".to_string()
                // Altrimenti, controlla se è un nome di nodo
                } else if let Some(node) = self.game_state.nodes.iter().find(|n| n.name.to_lowercase() == topic.to_lowercase()) {
                    // Spieghiamo le proprietà speciali di ogni nodo
                    match node.node_type {
                        crate::game::node::NodeType::Database => "--- NODO: Database [DB] ---\nContiene i dati critici. Se distrutto, la partita è persa. È vulnerabile a [SQL_Injection].".to_string(),
                        crate::game::node::NodeType::Firewall => "--- NODO: Firewall [FW] ---\nNodo difensivo. Subisce il 25% di danni in meno da tutti gli attacchi.".to_string(),
                        _ => format!("--- NODO: {} ---\nNessuna proprietà speciale nota.", node.name)
                    }
                } else {
                    format!("Help topic '{}' not found.", topic)
                }
            },
            // Se non c'è argomento, es. `help()`
            None => {
                "--- COMANDI DISPONIBILI ---\n  Fortify(NomeNodo)\n  Scan(NomeNodo)\n  help(topic)\n  quit\nPer info, usa help(comando) o help(NomeNodo).".to_string()
            }
        };
        // `extend` è come `push`, ma per aggiungere più elementi in una volta.
        // `split('\n')` divide la stringa in più righe.
        self.system_log.extend(help_text.split('\n').map(|s| s.to_string()));
    }

    fn with_mutable_node<F>(&mut self, node_name: &str, action: F)
    where
        F: FnOnce(&mut crate::game::node::Node, &mut Vec<String>),
    {
        // `.iter_mut().find()` ci permette di ottenere un riferimento *modificabile* al nodo.
        if let Some(node) = self.game_state.nodes.iter_mut().find(|n| n.name.eq_ignore_ascii_case(node_name)) {
            // Eseguiamo l'azione passata come closure.
            action(node, &mut self.system_log);
        } else {
            self.system_log.push(format!("Error: Node '{}' not found.", node_name));
        }
    }

    fn handle_fortify(&mut self, node_name: String) {
        // Definiamo il costo in AP del comando.
        const AP_COST: u8 = 2;
        if self.game_state.player_ap < AP_COST {
            self.system_log.push("Error: Not enough Action Points.".to_string());
            return; // Usciamo subito dalla funzione.
        }
        
        // Scaliamo gli AP
        self.game_state.player_ap -= AP_COST;

        self.with_mutable_node(&node_name, |node, log| {
            // `saturating_add` evita overflow: se il risultato è > 255, rimane 255.
            node.hp = node.hp.saturating_add(25);
            // `min` assicura che l'HP non superi il massimo.
            node.hp = std::cmp::min(node.hp, node.max_hp);

            // Aggiungiamo l'effetto Shield se non è già presente.
            use crate::game::node::StatusEffect;
            if !node.status_effects.contains(&StatusEffect::Shield) {
                node.status_effects.push(StatusEffect::Shield);
            }
            
            log.push(format!("[SUCCESS] Fortified node {}. HP is now {}.", node.name, node.hp));
        });
    }

    fn handle_scan(&mut self, node_name: String) {
        const AP_COST: u8 = 1;
        if self.game_state.player_ap < AP_COST {
            self.system_log.push("Error: Not enough Action Points.".to_string());
            return;
        }
        self.game_state.player_ap -= AP_COST;

        self.with_mutable_node(&node_name, |_node, log| {
            // Per ora, lo scan non trova mai nulla, ma la logica è qui.
            // TODO: Controllare se `_node` ha effetti nascosti.
            log.push(format!("[INFO] Scan of node {} complete. No hidden threats detected.", node_name));
        });
    }
}