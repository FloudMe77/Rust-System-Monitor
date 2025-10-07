use color_eyre::Result;

mod app;
mod models;
mod utils;

use app::{App, run};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new();
    
    let result = run(terminal, app);
    
    ratatui::restore();
    result
}
