use std::{io, thread, process, time::Duration};
use tui::{
    backend::{CrosstermBackend, Backend},
    widgets::{Block, Borders, Paragraph, List, ListItem, BarChart},
    layout::{Layout, Constraint, Direction, Rect, Alignment},
    text::{Span, Spans},
    style::{Style, Color, Modifier},
    Terminal,
    Frame,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use unicode_width::UnicodeWidthStr;
use chrono::{DateTime, Timelike};

use crate::discord::{self, User, Channel, Guild, Message, Messenger};

#[derive(PartialEq, Eq)]
enum Mode {
    Normal,
    User,
    Channel,
}

// A Channel or Guild
enum Location {
    Channel,
    Guild,
}

enum State {
    Idle,
    Working,
    Done,
}

pub struct App {
    pub token: String,
    // User whomst the token belongs to
    user: User,
    // Current Discord IDs of target user and channel
    target_user: User,
    target_chan: Channel,
    target_guil: Guild,
    // Switch between Channel and Guild
    target_loc: Location,
    // Current input mode of tui
    input_mode: Mode,
    // Active user input
    input_user: String,
    input_chan: String,
    // Messages that have been read
    messages: Vec<Message>,
    // Done indicator
    state: State,
    progress: (usize, u32)
}

impl App {
    pub fn new(token: String) -> reqwest::Result<Self> {
        Ok(App {
            user: discord::get_user("@me", &token)?,
            token: token,
            target_user: User::default(),
            target_chan: Channel::default(),
            target_guil: Guild::default(),
            target_loc: Location::Channel,
            input_mode: Mode::Normal,
            input_user: String::new(),
            input_chan: String::new(),
            messages: Vec::new(),
            state: State::Idle,
            progress: (0, 0),
        })
    }

    fn set_target_user(&mut self, id: &str) -> reqwest::Result<()> {
        self.target_user = discord::get_user(id, &self.token)?;
        Ok(())
    }

    fn set_target_chan(&mut self, id: &str) -> reqwest::Result<()> {
        self.target_chan = match discord::get_channel(id, &self.token) {
            Ok(l) => {
                self.target_loc = Location::Channel;
                l
            },
            Err(_) => {
                self.set_target_guil(id);
                Channel::default()
            },
        };
        Ok(())
    }

    fn set_target_guil(&mut self, id: &str) {
        self.target_guil = match discord::get_guild(id, &self.token) {
            Ok(l) => {
                self.target_loc = Location::Guild;
                l
            },
            Err(_) => Guild::default(),
        };
    }

    fn start<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        self.state = State::Working;
        self.messages.clear();
        terminal.draw(|f| draw(f, self)).unwrap();

        let mut messenger = match self.target_loc {
            Location::Channel => Messenger::new(
                self.token.clone(),
                self.target_user.id.clone(),
                self.target_chan.guild_id.clone(),
                Some(self.target_chan.id.clone()),
            ),
            Location::Guild => Messenger::new(
                self.token.clone(),
                self.target_user.id.clone(),
                self.target_guil.id.clone(),
                None,
            ),
        };

        while let Some(ms) = messenger.next() {
            self.progress = (messenger.offset, messenger.total_results);
            ms.into_iter().for_each(|m| {
                self.messages.push(m);
                terminal.draw(|f| draw(f, self));
            });

            if let Ok(true) = event::poll(Duration::from_secs(1)) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.code == KeyCode::Char('q') {
                        reset_terminal()?;
                        process::exit(0);
                    }
                }
            }
        }

        self.state = State::Done;

        Ok(())
    }
}

pub fn deploy(app: &mut App) -> io::Result<()> {
    let mut terminal = init_terminal()?;

    let res = run(&mut terminal, app);

    reset_terminal()?;

    if let Err(err) = res {
        println!("{}", err);
    }

    Ok(())
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.show_cursor()?;

    Ok(terminal)
}

fn reset_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('u') | KeyCode::Char('i') => {
                        app.input_mode = Mode::User;
                    },
                    KeyCode::Char('c') | KeyCode::Char('a') => {
                        app.input_mode = Mode::Channel;
                    }, KeyCode::Char('s') => { 
                        app.start(terminal)?;
                    },
                    KeyCode::Char('q') => {
                        return Ok(())
                    },
                    _ => {},
                },
                Mode::User => match key.code {
                    KeyCode::Enter => if !app.input_user.is_empty() {
                        app.messages.clear();
                        
                        let user_id = app.input_user.drain(..).collect::<String>();
                        if let Err(_) = app.set_target_user(&user_id) {
                            app.input_user.clear();
                            app.input_user = String::from("Invalid user id.");
                            thread::sleep(Duration::from_secs(2));
                            app.input_user.clear();
                        }
                        app.input_mode = Mode::Normal;
                    },
                    KeyCode::Esc => {
                        app.input_mode = Mode::Normal;
                    },
                    KeyCode::Backspace => {
                        app.input_user.pop();
                    },
                    KeyCode::Char(c) => {
                        app.input_user.push(c);
                    },
                    _ => {},
                },
                Mode::Channel => match key.code {
                    KeyCode::Enter => if !app.input_chan.is_empty() {
                        app.messages.clear();

                        let chan_id = app.input_chan.drain(..).collect::<String>();
                        if let Err(_) = app.set_target_chan(&chan_id) {
                            app.input_user.clear();
                            app.input_user = String::from("Invalid user id.");
                            thread::sleep(Duration::from_secs(2));
                            app.input_user.clear();
                        }
                        app.input_mode = Mode::Normal;
                    },
                    KeyCode::Esc => {
                        app.input_mode = Mode::Normal;
                    },
                    KeyCode::Backspace => {
                        app.input_chan.pop();
                    },
                    KeyCode::Char(c) => {
                        app.input_chan.push(c);
                    },
                    _ => {},
                }
            }
        }
    }
}

fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(75), 
                Constraint::Min(1),
            ]
            .as_ref()
        )
        .split(f.size());

    draw_top(f, chunks[0], app);
    draw_middle(f, chunks[1], app);
    draw_bottom(f, chunks[2], app);
}

fn draw_top<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ]
            .as_ref()
        )
        .split(area);

    let messages = List::new(app.messages
        .iter()
        .rev()
        .take(10)
        .map(|m| {
            let content = vec![
                Spans::from(Span::raw(format!("{}", m)))
            ];
            ListItem::new(content)
        })
        .collect::<Vec<ListItem>>())
        .block(Block::default().title("Messages").borders(Borders::ALL));
    f.render_widget(messages, chunks[0]);

    let target_user = if app.target_user.is_empty() {
        Span::styled("Awaiting target user input",
            Style::default().fg(Color::Red)
            .add_modifier(Modifier::SLOW_BLINK))
    } else {
        Span::styled(format!("{}", app.target_user),
            Style::default().fg(Color::Cyan)
            .add_modifier(Modifier::BOLD))
    };
    
    let target = match app.target_loc {
        Location::Channel => if app.target_chan.is_empty() {
            Span::styled("Awaiting target channel input",
                Style::default().fg(Color::Red)
                .add_modifier(Modifier::SLOW_BLINK))
        } else {
            Span::styled(format!("{}", app.target_chan),
                Style::default().fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD))
        },
        Location::Guild => if app.target_guil.is_empty() {
            Span::styled("Awaiting target channel input",
                Style::default().fg(Color::Red)
                .add_modifier(Modifier::SLOW_BLINK))
        } else {
            Span::styled(format!("{}", app.target_guil),
                Style::default().fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD))
        }
    };

    let info_msg = vec![
        Spans::from("Signed in as:"),
        Spans::from(Span::styled(format!("{}", app.user), 
            Style::default().fg(Color::LightMagenta)
            .add_modifier(Modifier::BOLD))),
        Spans::from("Target user ID:"),
        Spans::from(target_user),
        Spans::from("Target channel ID:"),
        Spans::from(target),
        Spans::from(match app.state {
            State::Idle => Span::styled("Idle", Style::default().fg(Color::Gray)),
            State::Working => Span::styled(
                format!("Working... ({}/{})", app.progress.0, app.progress.1), 
                Style::default().fg(Color::Yellow)),
            State::Done => Span::styled("Done!", Style::default().fg(Color::Green)),
        })
    ];

    let info = Paragraph::new(info_msg)
        .block(Block::default().title("Info").borders(Borders::ALL));
    f.render_widget(info, chunks[1]);
}

fn draw_middle<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    #[rustfmt::skip]
    let start = [
        ("12am", 0), ("1am", 0), ("2am", 0), ("3am", 0), 
        ("4am", 0), ("5am", 0), ("6am", 0), ("7am", 0), 
        ("8am", 0), ("9am", 0), ("10am", 0), ("11am", 0), 
        ("12pm", 0), ("1pm", 0), ("2pm", 0), ("3pm", 0), 
        ("4pm", 0), ("5pm", 0), ("6pm", 0), ("7pm", 0), 
        ("8pm", 0), ("9pm", 0), ("10pm", 0), ("11pm", 0),
    ];
    let data = app.messages.iter().fold(start, |mut acc, m| {
        if let Ok(t) = DateTime::parse_from_rfc3339(m.timestamp.as_str()) {
            acc[t.hour() as usize].1 += 1;
        }
        acc
    });
    // chart goes here
    let chart = BarChart::default()
        .block(Block::default().title("Data").borders(Borders::ALL))
        .data(&data)
        .bar_width(area.width / 23 - 5)
        .bar_gap(4)
        .bar_style(Style::default().fg(Color::LightMagenta))
        .value_style(Style::default().bg(Color::LightMagenta).add_modifier(Modifier::ITALIC));
    f.render_widget(chart, area);
}

fn draw_bottom<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    let (title, help_msg) = match app.input_mode {
        Mode::Normal => ("Normal",
            Spans::from(vec![
                Span::styled("i", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::styled("user edit mode", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" | "),
                Span::styled("a", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::styled("channel edit mode", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" | "),
                Span::styled("s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::styled("start", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" | "),
                Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::styled("exit", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ]),
        ),
        Mode::User | Mode::Channel => ("Insert",
            Spans::from(vec![
                Span::styled("esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::styled("normal mode", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" | "),
                Span::styled("enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::styled("confirm input", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
        ),
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(5, 8),
                Constraint::Ratio(3, 8),
            ]
            .as_ref()
        )
        .split(area);

    let help = Paragraph::new(help_msg)
        .block(Block::default()
            .title(title).borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[1]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref()
        )
        .split(chunks[0]);

    let input = Paragraph::new(app.input_user.as_ref())
        .block(Block::default().title("Target User").borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(input, chunks[0]);

    let input = Paragraph::new(app.input_chan.as_ref())
        .block(Block::default().title("Target Channel").borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(input, chunks[1]);

    match app.input_mode {
        Mode::Normal => {},
        Mode::User => f.set_cursor(
            chunks[0].x + app.input_user.width() as u16 + 1,
            chunks[0].y + 1,
        ),
        Mode::Channel => f.set_cursor(
            chunks[1].x + app.input_chan.width() as u16 + 1,
            chunks[1].y + 1,
        ),
    }
}
