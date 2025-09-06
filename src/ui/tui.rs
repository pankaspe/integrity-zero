// src/ui/tui.rs

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stderr};

// Un "type alias" per semplificare la scrittura del tipo Terminal.
pub type Tui = Terminal<CrosstermBackend<Stderr>>;

// Funzione per inizializzare il terminale.
pub fn init() -> io::Result<Tui> {
    // Entra in una "schermata alternativa" (non rovina la cronologia del tuo terminale).
    execute!(io::stderr(), EnterAlternateScreen)?;
    // Abilita la "modalità raw", che ci permette di leggere i tasti premuti
    // immediatamente, senza aspettare "Invio".
    enable_raw_mode()?;
    // Crea un nuovo terminale che disegnerà sull'output di errore standard.
    Terminal::new(CrosstermBackend::new(io::stderr()))
}

// Funzione per ripristinare il terminale.
pub fn restore() -> io::Result<()> {
    // Esce dalla schermata alternativa.
    execute!(io::stderr(), LeaveAlternateScreen)?;
    // Disabilita la modalità raw.
    disable_raw_mode()?;
    Ok(())
}