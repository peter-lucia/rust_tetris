use std::io;
use tui::{backend::CrosstermBackend, Terminal};
use unicode_width::UnicodeWidthStr;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::time::Duration;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders},
    Frame,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{List, ListItem, Paragraph},
};
use tui::widgets::canvas::{Canvas, Map, MapResolution, Rectangle};
use tui::layout::Rect;

// https://github.com/fdehau/tui-rs/blob/v0.18.0/examples/canvas.rs
/// App holds the state of the application
pub struct App {
    x: i32,
    y: i32,
    shape: Rectangle,
}

impl App {
    pub fn new() -> App {
        App {
            x: 0,
            y: 0,
            shape: Rectangle {
                x: 10.0,
                y: 30.0,
                width: 10.0,
                height: 10.0,
                color: Color::Yellow,
            },
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(4)
        .constraints(
            [
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ].as_ref()
        )
        .split(f.size());
    let mut block = Block::default().title("Tetris (q to quit)")
        .title_alignment(Alignment::Center).borders(Borders::ALL);
    f.render_widget(block, chunks[0]);
    let block = Block::default().title("Info")
        .title_alignment(Alignment::Center).borders(Borders::ALL);
    f.render_widget(block, chunks[1]);

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Title"))
        .paint(|ctx| {
            ctx.draw(&app.shape);
        })
        .x_bounds([10.0, 110.0])
        .y_bounds([10.0, 110.0]);
    f.render_widget(canvas, chunks[0]);
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {

        // update the terminal
        terminal.draw(|f| {
            ui(f, &app)
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    // quit
                    return Ok(());
                }
                KeyCode::Down => {
                    app.shape.y -= 1.0;
                }
                KeyCode::Up => {
                    app.shape.y += 1.0;
                }
                KeyCode::Right => {
                    app.shape.x += 1.0;
                }
                KeyCode::Left => {
                    app.shape.x -= 1.0;
                }
                _ => {}
            }
        }
    }
}