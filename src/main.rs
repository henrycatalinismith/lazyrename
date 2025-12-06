use crossterm::{
    event::{poll, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};
use std::{env, io, time::Duration};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: lazyrename <filename>");
        std::process::exit(1);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let src = args[1].clone();
    let dst = args[1].clone();

    loop {
        terminal.draw(|f| {
            render(f, &src, &dst);
        })?;

        if poll(Duration::from_millis(32))? {
            if let Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    return Ok(());
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn render(frame: &mut Frame, src: &str, dst: &str) {
    let size = frame.area();

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(size);

    let ui_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Length(2)])
        .flex(Flex::Center)
        .split(vchunks[0]);

    let src_para = Paragraph::new(Text::raw(src))
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Title")
                .border_type(BorderType::Rounded),
        );
    let dst_para = Paragraph::new(Text::raw(dst));

    let src_area = center(
        ui_chunks[0],
        Constraint::Length((src.len() as u16) + 4),
        Constraint::Length(3),
    );
    let dst_area = center(
        ui_chunks[1],
        Constraint::Length(dst.len() as u16),
        Constraint::Length(1),
    );

    frame.render_widget(src_para, src_area);
    frame.render_widget(dst_para, dst_area);
}
