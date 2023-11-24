use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use super::centered_rect;

#[derive(Debug)]
pub enum Type {
    Error,
    Informaton,
}

struct App<'a> {
    info_type: Type,
    header: &'a str,
    body: &'a str,
}

impl App<'_> {
    fn new<'a>(info_type: Type, header: &'a str, body: &'a str) -> App<'a> {
        App {
            info_type,
            header,
            body,
        }
    }
}

pub fn info_popup(info_type: Type, header: &str, body: &str) -> anyhow::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(info_type, header, body);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(());
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // let paragraph = Paragraph::new(app.body)
    //     .alignment(Alignment::Center)
    //     .wrap(Wrap { trim: true });

    let fg_color = match app.info_type {
        Type::Error => Color::Red,
        Type::Informaton => Color::Blue,
    };
    let block = Block::default()
        .title(app.header)
        .title_style(Style::default().bold())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .fg(fg_color);

    let content = Paragraph::new(app.body).block(Block::default().borders(Borders::ALL));

    let area = centered_rect(60, 15, size);
    f.render_widget(content, area);
    f.render_widget(block, area);
}
