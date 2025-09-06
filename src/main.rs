// src/main.rs

pub mod actions;
pub mod app;
pub mod game;
pub mod input;
pub mod ui;

use app::App;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ui::tui::init()?;
    
    let mut app = App::new();
    app.run(&mut terminal)?;

    ui::tui::restore()?;
    Ok(())
}