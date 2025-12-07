use clap::Parser;
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
use std::{
    fs, io,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(name="lazyrename", version, about, long_about = None)]
struct Args {
    /// Helix mode
    #[arg(long)]
    helix: bool,

    /// Zellij mode
    #[arg(long)]
    zellij: bool,

    /// Input filename (required positional argument)
    filename: String,
}

struct ThemeEntry {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
}

struct WidgetTheme {
    pub block: ThemeEntry,
    pub text: ThemeEntry,
}

struct StatusTheme {
    pub idle: ThemeEntry,
    pub error: ThemeEntry,
}

struct Theme {
    pub src: WidgetTheme,
    pub dst: WidgetTheme,
    pub status: StatusTheme,
}

enum StatusLevel {
    Idle,
    Error,
}

struct Status {
    pub text: String,
    pub level: StatusLevel,
}

impl StatusTheme {
    fn get(&self, level: &StatusLevel) -> &ThemeEntry {
        match level {
            StatusLevel::Idle => &self.idle,
            StatusLevel::Error => &self.error,
        }
    }
}

struct State {
    src: String,
    dst: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    if args.filename.is_empty() {
        eprintln!("Usage: lazyrename <filename>");
        std::process::exit(1);
    }

    let theme = Theme {
        src: WidgetTheme {
            block: ThemeEntry {
                bg: None,
                fg: Some(Color::Blue),
            },
            text: ThemeEntry {
                bg: None,
                fg: Some(Color::Black),
            },
        },
        dst: WidgetTheme {
            block: ThemeEntry {
                bg: None,
                fg: Some(Color::White),
            },
            text: ThemeEntry {
                bg: None,
                fg: Some(Color::White),
            },
        },
        status: StatusTheme {
            idle: ThemeEntry {
                bg: None,
                fg: Some(Color::Black),
            },
            error: ThemeEntry {
                bg: None,
                fg: Some(Color::Red),
            },
        },
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = State {
        src: args.filename.clone(),
        dst: args.filename.clone(),
    };

    let mut status = Status {
        text: "Press Enter to rename, Esc to cancel".to_string(),
        level: StatusLevel::Idle,
    };

    loop {
        terminal.draw(|f| {
            render(f, &state, &status, &theme);
        })?;

        if poll(Duration::from_millis(32))? {
            if let Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => state.dst.push(c),
                    KeyCode::Backspace => {
                        state.dst.pop();
                    }

                    KeyCode::Enter => match rename(&state, &args) {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            status.text = format!("{}", e);
                            status.level = StatusLevel::Error;
                        }
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

fn render(frame: &mut Frame, state: &State, status: &Status, theme: &Theme) {
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
    let src_para = filename_widget("Current filename", &state.src, &theme.src);
    let dst_para = filename_widget("New filename", &state.dst, &theme.dst);

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

    frame.render_widget(src_para, src_area);
    frame.render_widget(dst_para, dst_area);
    frame.render_widget(status_widget(status, &theme.status), ui_chunks[2]);
}

fn filename_widget<'a>(label: &'a str, filename: &'a str, theme: &'a WidgetTheme) -> Paragraph<'a> {
    let mut block_style = Style::default();
    let mut text_style = Style::default();
    if theme.block.fg.is_some() {
        block_style = block_style.fg(theme.block.fg.unwrap());
    }
    if theme.block.bg.is_some() {
        block_style = block_style.bg(theme.block.bg.unwrap());
    }
    if theme.text.fg.is_some() {
        text_style = text_style.fg(theme.text.fg.unwrap());
    }
    if theme.text.bg.is_some() {
        text_style = text_style.bg(theme.text.bg.unwrap());
    }
    Paragraph::new(Text::raw(filename)).style(text_style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(block_style)
            .title(label),
    )
}

fn status_widget<'a>(status: &'a Status, theme: &'a StatusTheme) -> Paragraph<'a> {
    let mut style = Style::default();
    let theme_entry = theme.get(&status.level);
    if theme_entry.fg.is_some() {
        style = style.fg(theme_entry.fg.unwrap());
    }
    if theme_entry.bg.is_some() {
        style = style.bg(theme_entry.bg.unwrap());
    }
    Paragraph::new(Text::raw(status.text.clone()))
        .style(style)
        .alignment(ratatui::layout::Alignment::Center)
}

fn rename(state: &State, args: &Args) -> io::Result<()> {
    if state.src == state.dst {
        return Ok(());
    }
    let dst_path = Path::new(&state.dst);
    if let Some(parent) = dst_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    if args.helix && args.zellij {
        let helix_command = format!(":move \"{}\"", state.dst);
        zellij_action(&["toggle-floating-panes"]);
        zellij_action(&["write", "27"]);
        zellij_action(&["write-chars", &helix_command]);
        zellij_action(&["write", "13"]);
    } else {
        fs::rename(&state.src, &state.dst)?;
    }
    Ok(())
}

fn zellij_action(args: &[&str]) {
    let _ = Command::new("zellij")
        .arg("action")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
