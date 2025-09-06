// src/ui/renderer.rs

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph}, 
    style::Color,
    text::{Line, Span},
};

pub fn render_main_ui(frame: &mut Frame, app: &App) {
    // --- LAYOUT ---
    // Definiamo un layout a 3 righe:
    // 1. La Status Bar in alto (altezza fissa).
    // 2. L'area principale, che occuperà tutto lo spazio rimanente.
    // 3. La Input Bar in basso (altezza fissa).
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Per la Status Bar
            Constraint::Min(0),    // Area principale, prende lo spazio che avanza
            Constraint::Length(3), // Per la Input Bar
        ])
        .split(frame.area()); // Applichiamo il layout all'intera area del frame

    // A sua volta, dividiamo l'area principale in due colonne:
    // 1. La Network View a sinistra (55% della larghezza).
    // 2. Il System Log a destra (45% della larghezza).
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_layout[1]); // Applichiamo questo layout al secondo pezzo del layout precedente

    // --- WIDGETS ---
    render_status_bar(frame, app, main_layout[0]);
    render_network_view(frame, app, main_chunks[0]);
    render_system_log(frame, app, main_chunks[1]);
    render_input_bar(frame, app, main_layout[2]);
}

// Funzioni "helper" per disegnare ogni pezzo dell'interfaccia.
// Questo mantiene il codice pulito e organizzato.

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    // Creiamo un layout orizzontale per dividere la status bar in 3 sezioni
    let status_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Per l'integrità
            Constraint::Percentage(20), // Per il turno
            Constraint::Percentage(40), // Per gli AP
        ])
        .split(area);

    // --- Widget Integrità ---
    let integrity_gauge = Gauge::default()
        .block(Block::default().title("System Integrity"))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(app.game_state.global_integrity.into()); // Convertiamo u8 in u16
    frame.render_widget(integrity_gauge, status_layout[0]);

    // --- Widget Turno ---
    let turn_text = format!("Turn: {}", app.game_state.turn_count);
    let turn_paragraph = Paragraph::new(turn_text)
        .block(Block::default().title("Game Turn"))
        .alignment(Alignment::Center);
    frame.render_widget(turn_paragraph, status_layout[1]);

    // --- Widget AP ---
    let ap_text = format!("AP: {} / 10", app.game_state.player_ap);
    let ap_paragraph = Paragraph::new(ap_text)
        .block(Block::default().title("Action Points"))
        .alignment(Alignment::Right);
    frame.render_widget(ap_paragraph, status_layout[2]);
}

fn render_network_view(frame: &mut Frame, app: &App, area: Rect) {
    let network_block = Block::default().borders(Borders::ALL).title("Network View");
    // Prima disegniamo il blocco vuoto che farà da contenitore.
    frame.render_widget(network_block, area);

    // Creiamo un layout a griglia 4x4 all'interno dell'area della Network View.
    // Dobbiamo togliere i bordi dall'area per non disegnare sopra di essi.
    let inner_area = area.inner(Margin { vertical: 1, horizontal: 1 });

    let grid_layout_vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(25); 4]) // 4 righe uguali
        .split(inner_area);

    let mut grid_cells: Vec<Vec<Rect>> = Vec::new();
    for row_area in grid_layout_vertical.iter() {
        let row_cells = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25); 4]) // 4 colonne uguali
            .split(*row_area);
        grid_cells.push(row_cells.to_vec());
    }

    // Ora che abbiamo le coordinate di ogni cella, disegniamo i nodi.
    for node in &app.game_state.nodes {
        let (x, y) = (node.pos.0 as usize, node.pos.1 as usize);
        let cell_area = grid_cells[y][x]; // Prendiamo l'area della cella giusta

        // Determiniamo il colore del bordo in base all'HP
        let border_color = match node.hp {
            hp if hp > 70 => Color::Green,
            hp if hp > 30 => Color::Yellow,
            _ => Color::Red,
        };

        let node_text = vec![
            Line::from(Span::styled(format!("[{}] {}", node.id, node.name), Style::default().bold())),
            Line::from(format!("HP: {}/{}", node.hp, node.max_hp)),
        ];

        let node_paragraph = Paragraph::new(node_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );
        
        frame.render_widget(node_paragraph, cell_area);
    }
}

fn render_system_log(frame: &mut Frame, app: &App, area: Rect) {
    // Convertiamo la nostra Vec<String> in una Vec<Line> che Paragraph può usare.
    let log_lines: Vec<Line> = app.system_log.iter().map(|s| Line::from(s.clone())).collect();
    
    let log_paragraph = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title("System Log"))
        // Permette al testo di andare a capo se è più lungo della larghezza del blocco.
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(log_paragraph, area);
}

fn render_input_bar(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = format!("> {}", app.input_text);
    let input_paragraph = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title("Input Command"));
    
    frame.render_widget(input_paragraph, area);

    // Rendiamo visibile il cursore lampeggiante.
    // La sua posizione è dopo il prompt `> ` e il testo inserito.
    frame.set_cursor(
        area.x + 2 + app.input_text.len() as u16,
        area.y + 1
    );
}