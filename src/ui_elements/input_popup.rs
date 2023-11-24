use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use super::centered_rect;

struct App<'a> {
    header: &'a str,
    body: &'a str,
    msg: Input,
}

impl App<'_> {
    fn new<'a>(header: &'a str, body: &'a str) -> App<'a> {
        App {
            header,
            body,
            msg: Input::default(),
        }
    }
}

pub fn input_popup(header: &str, body: &str) -> anyhow::Result<String> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(header, body);
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
        Err(err.into())
    } else {
        Ok(res.unwrap())
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> anyhow::Result<String> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => return Ok(app.msg.to_string()),
                    KeyCode::Esc => return Err(anyhow::anyhow!("Exited.")),
                    _ => {
                        app.msg.handle_event(&Event::Key(key));
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .constraints([Constraint::Max(10), Constraint::Min(0)])
        .split(size);

    let mut body = app.body.split('\n').map(|msg| Line::from(msg)).collect();
    let mut text = vec![Line::from(vec![Span::styled(
        "Press Enter to confirm, Escape to quit.",
        Style::default().slow_blink().bold(),
    )])];
    text.append(&mut body);

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, chunks[0]);

    let block = Block::default()
        .title(app.header)
        .title_style(Style::default().bold())
        .borders(Borders::ALL)
        .fg(Color::Yellow);

    let input = Paragraph::new(app.msg.value()).block(Block::default().borders(Borders::ALL));

    let area = centered_rect(60, 20, size);
    f.set_cursor(area.x + ((app.msg.visual_cursor()) as u16 + 1), area.y + 1);

    f.render_widget(input, area);
    f.render_widget(block, area);
}
