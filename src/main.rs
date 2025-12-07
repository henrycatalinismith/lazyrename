use clap::Parser;
use ratatui::crossterm::{
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
use tui_textarea::{Input, TextArea};

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
    pub input: WidgetTheme,
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

struct State<'a> {
    src: String,
    input: TextArea<'a>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    if args.filename.is_empty() {
        eprintln!("Usage: lazyrename <filename>");
        std::process::exit(1);
    }

    let theme = Theme {
        input: WidgetTheme {
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
        input: TextArea::default(),
    };

    state.input.insert_str(args.filename.clone());
    // state
    //     .input
    //     .set_cursor(args.filename.chars().count().try_into().unwrap(), false);

    let mut status = Status {
        text: "Press Enter to rename, Esc to cancel".to_string(),
        level: StatusLevel::Idle,
    };

    loop {
        terminal.draw(|f| {
            render(f, &mut state, &status, &theme);
        })?;

        if poll(Duration::from_millis(32))? {
            if let Event::Key(key) = ratatui::crossterm::event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => match rename(&state, &args) {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            status.text = format!("{}", e);
                            status.level = StatusLevel::Error;
                        }
                    },
                    _ => {
                        let input = Input::from(key);
                        let _ = state.input.input(input);
                    }
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

fn render(frame: &mut Frame, state: &mut State, status: &Status, theme: &Theme) {
    let size = frame.area();

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(size);

    let ui_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .flex(Flex::Center)
        .split(vchunks[0]);

    let block_width = std::cmp::min(size.width, 128);

    let input_area = center(
        ui_chunks[0],
        Constraint::Length(block_width),
        Constraint::Length(3),
    );

    let mut block_style = Style::default();
    if theme.input.block.fg.is_some() {
        block_style = block_style.fg(theme.input.block.fg.unwrap());
    }
    if theme.input.block.bg.is_some() {
        block_style = block_style.bg(theme.input.block.bg.unwrap());
    }

    let mut text_style = Style::default();
    if theme.input.text.fg.is_some() {
        text_style = text_style.fg(theme.input.text.fg.unwrap());
    }
    if theme.input.text.bg.is_some() {
        text_style = text_style.bg(theme.input.text.bg.unwrap());
    }
    state.input.set_style(text_style);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(block_style)
        .title("New filename");

    frame.render_widget(input_block.clone(), input_area);
    frame.render_widget(input_block.clone(), input_area);
    frame.render_widget(status_widget(status, &theme.status), ui_chunks[1]);

    let inner = input_block.inner(input_area);
    frame.render_widget(&state.input, inner);
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
    let dst: String = state.input.lines().join("\n");
    if state.src == dst {
        return Ok(());
    }
    let dst_path = Path::new(&dst);
    if let Some(parent) = dst_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    if args.helix && args.zellij {
        let helix_command = format!(":move \"{}\"", dst);
        zellij_action(&["toggle-floating-panes"]);
        zellij_action(&["write", "27"]);
        zellij_action(&["write-chars", &helix_command]);
        zellij_action(&["write", "13"]);
    } else {
        fs::rename(&state.src, &dst)?;
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
