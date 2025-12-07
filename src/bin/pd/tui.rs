use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use parallel_downloader::ipc::{Command, Response};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use std::{io, time::Duration};

use crate::client::send_command_raw;

pub async fn start_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        println!("TUI Error: {:?}", e);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    loop {
        let status_response = send_command_raw(Command::Status).await;

        let jobs = match status_response {
            Ok(Response::StatusList(jobs)) => jobs,
            _ => vec![],
        };

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // Header
            let header = Paragraph::new("Parallel Downloader Dashboard")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(header, chunks[0]);

            let rows: Vec<Row> = jobs
                .iter()
                .map(|job| {
                    let progress_text = format!("{}%", job.progress_percent);

                    let status_style = match job.state.as_str() {
                        "Done" => Style::default().fg(Color::Green),
                        s if s.starts_with("Error") || s.starts_with("Failed") => {
                            Style::default().fg(Color::Red)
                        }
                        _ => Style::default().fg(Color::Yellow),
                    };

                    Row::new(vec![
                        Cell::from(job.id.to_string()),
                        Cell::from(job.filename.clone()),
                        Cell::from(progress_text),
                        Cell::from(job.state.clone()).style(status_style),
                    ])
                })
                .collect();

            let widths = [
                Constraint::Length(5),
                Constraint::Percentage(40),
                Constraint::Length(10),
                Constraint::Percentage(40),
            ];

            let table = Table::new(rows, widths)
                .header(
                    Row::new(vec!["ID", "Filename", "Progress", "State"])
                        .style(Style::default().fg(Color::Yellow)),
                )
                .block(
                    Block::default()
                        .title("Active Downloads")
                        .borders(Borders::ALL),
                );

            f.render_widget(table, chunks[1]);

            // Footer
            let footer = Paragraph::new("Press 'q' to quit TUI (Daemon keeps running)")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
            && key.code == KeyCode::Char('q')
        {
            return Ok(());
        }
    }
}
