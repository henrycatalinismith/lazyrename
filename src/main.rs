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
use std::{env, fs, io, time::Duration};

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
    let mut dst = args[1].clone();
    let mut status = "Press Enter to rename, Esc to cancel".to_string();

    loop {
        terminal.draw(|f| {
            render(f, &src, &dst, &status);
        })?;

        if poll(Duration::from_millis(32))? {
            if let Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => dst.push(c), // add typed char
                    KeyCode::Backspace => {
                        dst.pop();
                    }

                    KeyCode::Enter => match rename(&src, &dst) {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => status = format!("{}", e),
                    },
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

fn render(frame: &mut Frame, src: &str, dst: &str, status: &str) {
    let size = frame.area();

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(size);

    let ui_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .split(vchunks[0]);

    let block_width = std::cmp::min(size.width, 128);
    let src_para = filename_block("Current filename", src, Color::Yellow);
    let dst_para = filename_block("New filename", dst, Color::White);

    let src_area = center(
        ui_chunks[0],
        Constraint::Length(block_width),
        Constraint::Length(3),
    );
    let dst_area = center(
        ui_chunks[1],
        Constraint::Length(block_width),
        Constraint::Length(3),
    );

    let status_line = Paragraph::new(Text::raw(status))
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(src_para, src_area);
    frame.render_widget(dst_para, dst_area);
    frame.render_widget(status_line, ui_chunks[2]);
}

fn filename_block<'a>(label: &'a str, filename: &'a str, fg: Color) -> Paragraph<'a> {
    Paragraph::new(Text::raw(filename))
        .style(Style::default().fg(fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(label),
        )
}

fn rename(src: &str, dst: &str) -> io::Result<()> {
    if src == dst {
        return Ok(());
    }
    fs::rename(src, dst)?;
    Ok(())
}
