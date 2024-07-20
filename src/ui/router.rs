use ratatui::crossterm::event::KeyCode;

use ratatui::crossterm::event::Event;

use ratatui::style::Stylize;
use ratatui::widgets::List;

use ratatui::widgets::ListItem;

use ratatui::widgets::Borders;

use ratatui::widgets::Block;

use ratatui::layout::Rect;

use ratatui::terminal::Frame;

use ratatui::style::Color;

use ratatui::text::Span;

#[derive(Debug, Clone, Copy)]
pub(crate) enum RouteType {
    Pop,
    Push,
    Quit,
}

#[derive(Debug, Clone)]
pub(crate) struct Route {
    pub(crate) rtype: RouteType,
    pub(crate) label: String,
    pub(crate) name: String,
    pub(crate) span: Span<'static>,
}

impl Route {
    pub(crate) fn new(rtype: RouteType, name: &str, label: &str) -> Self {
        Self {
            rtype,
            label: label.to_string(),
            span: match rtype {
                RouteType::Push => Span::from(label.to_string()).fg(Color::Blue),
                _ => Span::from(label.to_string()).fg(Color::Red),
            },
            name: name.to_string(),
        }
    }

    pub(crate) fn with_span(mut self, span: Span<'static>) -> Self {
        self.span = span.content(self.label.clone());
        self
    }
}

#[derive(Debug)]
pub(crate) struct Router {
    pub(crate) active_tabs: Vec<Route>,
    pub(crate) active_tab: usize,
    pub(crate) tabs_stack: Vec<(usize, Vec<Route>)>,
}

impl Router {
    pub(crate) fn new(init_tabs: Vec<Route>) -> Self {
        Self {
            active_tabs: init_tabs,
            active_tab: 0,
            tabs_stack: vec![],
        }
    }

    pub(crate) fn current_tab(&self) -> &Route {
        self.active_tabs.get(self.active_tab).unwrap()
    }

    pub(crate) fn current_path(&self) -> String {
        let r: Vec<String> = self
            .tabs_stack
            .iter()
            .map(|v: &(usize, Vec<Route>)| v.1[v.0].name.clone())
            .collect();
        if r.is_empty() {
            "".to_string()
        } else {
            format!("/{}", r.join("/"))
        }
    }

    pub(crate) fn push(&mut self, tabs: Vec<Route>, active: usize) {
        self.tabs_stack
            .push((self.active_tab, self.active_tabs.drain(..).collect()));
        self.active_tabs = tabs;
        self.active_tab = active;
    }

    pub(crate) fn pop(&mut self) {
        if let Some((active_tab, active_tabs)) = self.tabs_stack.pop() {
            self.active_tab = active_tab;
            self.active_tabs = active_tabs;
        }
    }

    pub(crate) fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::new().borders(Borders::ALL).title("Mongobar");
        let items: Vec<ListItem> = self
            .active_tabs
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i == self.active_tab {
                    ListItem::new(t.span.clone()).bg(Color::DarkGray)
                } else {
                    ListItem::new(t.span.clone())
                }
            })
            .collect();
        let list = List::new(items).block(block);

        f.render_widget(list, area);
    }

    pub(crate) fn event(&mut self, event: &Event) -> EventType {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('q') {
                return EventType::Quit;
            }
            if key.code == KeyCode::Up {
                if self.active_tab > 0 {
                    self.active_tab -= 1;
                } else {
                    self.active_tab = self.active_tabs.len() - 1;
                }
            } else if key.code == KeyCode::Down {
                if self.active_tab < self.active_tabs.len() - 1 {
                    self.active_tab += 1;
                } else {
                    self.active_tab = 0;
                }
            } else if key.code == KeyCode::Enter {
                let cp = self.current_path();
                let ctab = self.current_tab();
                let cptab = cp + "/" + ctab.name.as_str();
                return EventType::Click(cptab, ctab.rtype, key.code);
            } else if key.code == KeyCode::Left {
                let cp = self.current_path();
                let ctab = self.current_tab();
                let cptab = cp + "/" + ctab.name.as_str();
                return EventType::Click(cptab, ctab.rtype, key.code);
            } else if key.code == KeyCode::Right {
                let cp = self.current_path();
                let ctab = self.current_tab();
                let cptab = cp + "/" + ctab.name.as_str();
                return EventType::Click(cptab, ctab.rtype, key.code);
            }
        }

        return EventType::Inner;
    }
}

#[derive(Debug, Clone)]
pub(crate) enum EventType {
    Quit,
    Click(String, RouteType, KeyCode),
    Inner,
}
