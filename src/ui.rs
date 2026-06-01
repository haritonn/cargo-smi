use crate::{app::AppState, error::Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::{
    io::{self, stdout},
    time::Duration,
};

pub fn run_tui(state: &mut AppState) -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_tui_loop(&mut terminal, state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
) -> Result<()> {
    state.refresh_selected();
    loop {
        if state.should_refresh() {
            state.refresh_selected();
        }
        terminal.draw(|frame| {
            let area = frame.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(area);
            let header = Paragraph::new("cargo-smi").block(Block::default().borders(Borders::ALL));
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(1)])
                .split(chunks[1]);
            let selected_pos = state.selected_pos();
            let items: Vec<ListItem> = state
                .gpu_entries()
                .iter()
                .enumerate()
                .map(|(pos, entry)| {
                    let prefix = if Some(pos) == selected_pos { "> " } else { " " };
                    ListItem::new(format!(
                        "{prefix}{}: {}",
                        entry.device.idx, entry.device.name
                    ))
                })
                .collect();
            let gpu_list =
                List::new(items).block(Block::default().title("GPUs").borders(Borders::ALL));
            let body_text = match state.selected_gpu_mut() {
                Ok(entry) => {
                    let stats = match &entry.stats {
                        Some(stats) => stats.to_string(),
                        None => "No stats yet".to_owned(),
                    };
                    format!(
                        "GPU {}: {}\n\n{}",
                        entry.device.idx, entry.device.name, stats
                    )
                }
                Err(err) => format!("Error: {err}"),
            };
            let body = Paragraph::new(body_text)
                .block(Block::default().title("Stats").borders(Borders::ALL));

            let footer_text = match state.last_error() {
                Some(err) => format!("q/Esc quit | j/↓ next | k/↑ prev | r refresh | Error: {err}"),
                None => "q/Esc quit | j/↓ next | k/↑ prev | r refresh".to_owned(),
            };
            let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));

            frame.render_widget(header, chunks[0]);
            frame.render_widget(gpu_list, body_chunks[0]);
            frame.render_widget(body, body_chunks[1]);
            frame.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => state.quit(),
                KeyCode::Char('j') | KeyCode::Down => {
                    state.select_next();
                    state.refresh_selected();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    state.select_prev();
                    state.refresh_selected();
                }
                KeyCode::Char('r') => {
                    state.refresh_selected();
                }

                _ => {}
            }
        }

        if state.should_quit() {
            break;
        }
    }

    Ok(())
}
