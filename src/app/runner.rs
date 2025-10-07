use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};
use ratatui::Terminal;

use crate::models;
use super::App;

pub fn run(mut terminal: Terminal<impl ratatui::backend::Backend>, mut app: App) -> Result<()> {
    let mut last_update = Instant::now();
    // przetwarzam uprzednio załadowane dane
    app.sort_data();
    app.save_history_data();

    // główna pętla programu
    loop {
        // rysowanie
        terminal.draw(|frame| app.draw(frame))?;

        // obsługa stopu
        if !app.stop {
            if last_update.elapsed() >= Duration::from_millis(models::INTERVAL) {
                app.update_data();
                last_update = Instant::now();
            }
        }
        // czekanie + obsługa przycisków
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if handle_key_event(&mut app, key.code, key.modifiers.contains(KeyModifiers::SHIFT)) {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyCode, shift_pressed: bool) -> bool {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Down if shift_pressed => {
            app.reverse_sort = true;
            app.sort_data();
            false
        }
        KeyCode::Up if shift_pressed => {
            app.reverse_sort = false;
            app.sort_data();
            false
        }
        KeyCode::Down => {
            app.next_row();
            false
        }
        KeyCode::Up => {
            app.previous_row();
            false
        }
        KeyCode::Left if shift_pressed => {
            app.sort_tag = app.sort_tag.prev();
            app.sort_data();
            false
        }
        KeyCode::Right if shift_pressed => {
            app.sort_tag = app.sort_tag.next();
            app.sort_data();
            false
        }
        KeyCode::Right => {
            app.next_column();
            false
        }
        KeyCode::Left => {
            app.previous_column();
            false
        }
        KeyCode::Tab => {
            app.plot_cpu = true;
            false
        }
        KeyCode::Enter => {
            if let Some(selected) = app.state.selected() {
                if let Some(proc) = app.items.get(selected) {
                    app.chart_col = app.selected_column;
                    app.chart_pid = proc.pid;
                    app.plot_cpu = false;
                }
            }
            false
        }
        KeyCode::Char(' ') => {
            app.stop = !app.stop;
            false
        }
        _ => false,
    }
} 