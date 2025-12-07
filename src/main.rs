use clap::Parser;
use rat_widget::text_input::{handle_events, TextInput, TextInputState};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};
use ratatui::{
    crossterm::{
        event::{poll, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Position,
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

struct State {
    src: String,
    input: TextInputState,
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
        input: TextInputState::new(),
    };

    state.input.set_value(args.filename.clone());

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
                        handle_events(&mut state.input, true, &Event::Key(key));
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
    let mut text_style = Style::default();
    if theme.input.block.fg.is_some() {
        block_style = block_style.fg(theme.input.block.fg.unwrap());
    }
    if theme.input.block.bg.is_some() {
        block_style = block_style.bg(theme.input.block.bg.unwrap());
    }
    if theme.input.text.fg.is_some() {
        text_style = text_style.fg(theme.input.text.fg.unwrap());
    }
    if theme.input.text.bg.is_some() {
        text_style = text_style.bg(theme.input.text.bg.unwrap());
    }

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(block_style)
        .title("New filename");

    frame.render_widget(input_block.clone(), input_area);
    frame.render_widget(status_widget(status, &theme.status), ui_chunks[1]);

    let inner = input_block.inner(input_area);
    let textinput_widget = TextInput::new().style(text_style);
    frame.render_stateful_widget(textinput_widget, inner, &mut state.input);
    let cursor_x = inner.x + state.input.cursor() as u16;
    let cursor_y = inner.y;
    frame.set_cursor_position(Position::from((cursor_x, cursor_y)));
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
    let dst: String = state.input.value::<String>().to_string();
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
