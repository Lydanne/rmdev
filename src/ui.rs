use std::{
    error::Error,
    io,
    sync::{Arc, Mutex},
};

use ratatui::{
    crossterm::{
        self,
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::*,
};
use style::palette::tailwind;
use unicode_width::UnicodeWidthStr;

use crate::command::clear::ScanRow;

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::RED,
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
];
const INFO_TEXT: &str = "(Esc) quit | (↑) move up | (↓) move down | (Enter) clear all cache";

const ITEM_HEIGHT: usize = 4;

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

pub struct UI {
    pub rows: Arc<Mutex<Vec<ScanRow>>>,
}

struct App {
    state: TableState,
    longest_item_lens: Vec<usize>,
    scroll_state: ScrollbarState,
    colors: TableColors,
    color_index: usize,
    ui: UI,
}

impl App {
    fn new(ui: UI) -> Self {
        let scan_rows = ui.rows.clone();
        Self {
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(scan_rows.clone()),
            scroll_state: ScrollbarState::new(0 * ITEM_HEIGHT),
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            ui,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.ui.rows.lock().unwrap().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state =
            ScrollbarState::new((self.ui.rows.clone().lock().unwrap().len() - 1) * ITEM_HEIGHT)
                .position(i * ITEM_HEIGHT);
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.ui.rows.lock().unwrap().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state =
            ScrollbarState::new((self.ui.rows.clone().lock().unwrap().len() - 1) * ITEM_HEIGHT)
                .position(i * ITEM_HEIGHT);
    }

    pub fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
    }

    pub fn previous_color(&mut self) {
        let count = PALETTES.len();
        self.color_index = (self.color_index + count - 1) % count;
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }
}

/// 2 异常退出 1 退出， 0 是继续
pub fn boot(ui: UI) -> Result<usize, Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(ui);
    let res = run_app(&mut terminal, app);
    let code = if let io::Result::Ok(code) = res {
        code
    } else {
        2
    };

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
        return Ok(2);
    }

    Ok(code)
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<usize> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                use KeyCode::*;
                match key.code {
                    Char('q') | Esc => return Ok(1),
                    Char('y') | Enter => return Ok(0),
                    Char('j') | Down => app.next(),
                    Char('k') | Up => app.previous(),
                    Char('l') | Right => app.next_color(),
                    Char('h') | Left => app.previous_color(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let rects = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(f.size());

    app.set_colors();

    render_header(f, app, rects[0]);

    render_table(f, app, rects[1]);

    render_scrollbar(f, app, rects[1]);

    render_footer(f, app, rects[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new("  🚀 Rmdev (https://github.com/WumaCoder/rmdev 🌟)")
        .style(
            Style::new()
                .fg(app.colors.header_fg)
                .bg(app.colors.header_bg),
        )
        // .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(app.colors.footer_border_color)),
        );
    f.render_widget(header, area);
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = ScanRow::ref_head()
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
    let scan_rows = app.ui.rows.lock().unwrap();
    let rows = scan_rows.iter().enumerate().map(|(i, scan_row)| {
        let color = match i % 2 {
            0 => app.colors.normal_row_color,
            _ => app.colors.alt_row_color,
        };
        let item = scan_row.ref_data();
        item.into_iter()
            .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
            .collect::<Row>()
            .style(Style::new().fg(app.colors.row_fg).bg(color))
            .height(3)
    });
    let bar = " █ ";
    let t = Table::new(
        rows,
        [
            Constraint::Percentage(25 as u16),
            Constraint::Max(10 as u16),
            Constraint::Max(10 as u16),
            Constraint::Min(10 as u16),
        ], // app.longest_item_lens
           //     .iter()
           //     .map(|w| Constraint::Min(*w as u16)),
    )
    .header(header)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
    .bg(app.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);
    f.render_stateful_widget(t, area, &mut app.state);
}

fn constraint_len_calculator(items: Arc<Mutex<Vec<ScanRow>>>) -> Vec<usize> {
    let mut widths = ScanRow::ref_head().map(UnicodeWidthStr::width);

    for (_i, row) in items.lock().unwrap().iter().enumerate() {
        let row_widths = row.ref_data().map(|s| UnicodeWidthStr::width(s.as_str()));
        for (_j, (width, row_width)) in widths.iter_mut().zip(row_widths).enumerate() {
            if *width < row_width {
                *width = row_width;
            }
        }
    }
    widths.to_vec()
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let info_footer = Paragraph::new(Line::from(format!(
        "{INFO_TEXT} ({})",
        app.ui.rows.lock().unwrap().len()
    )))
    .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
    .centered()
    .block(
        Block::bordered()
            .border_type(BorderType::Double)
            .border_style(Style::new().fg(app.colors.footer_border_color)),
    );
    f.render_widget(info_footer, area);
}
