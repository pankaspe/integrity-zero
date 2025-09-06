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
            Command::Help(arg) => {
                let help_text = match arg {
                    Some(topic) => format!("Help for topic: {}", topic), // Logica da implementare
                    None => "General help: \nAvailable commands: \n  help(topic)\n  fortify(node)\n  scan(node)\n  q / quit".to_string(),
                };
                self.system_log.push(help_text);
            }
            Command::Fortify(node_name) => {
                self.system_log.push(format!("[TODO] Fortifying node: {}", node_name));
            }
            Command::Scan(node_name) => {
                self.system_log.push(format!("[TODO] Scanning node: {}", node_name));
            }
            Command::Quit => {
                self.exit = true;
            }
            Command::Invalid => {
                self.system_log.push("Error: Invalid command.".to_string());
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