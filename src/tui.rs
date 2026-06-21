use crate::model::Reclaimable;
use crate::report;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{Frame, Terminal};
use std::cmp::Ordering;
use std::io::stdout;

#[derive(Clone, Copy)]
enum Sort {
    Size,
    Age,
    Name,
}

impl Sort {
    fn next(self) -> Sort {
        match self {
            Sort::Size => Sort::Age,
            Sort::Age => Sort::Name,
            Sort::Name => Sort::Size,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Sort::Size => "size",
            Sort::Age => "age",
            Sort::Name => "name",
        }
    }
}

/// Picker state. `order` holds indices into `items` in display order (filtered +
/// sorted); `selected` is indexed by the original item index.
struct App<'a> {
    items: &'a [Reclaimable],
    selected: Vec<bool>,
    order: Vec<usize>,
    state: ListState,
    sort: Sort,
    filtering: bool,
    query: String,
}

impl<'a> App<'a> {
    fn new(items: &'a [Reclaimable]) -> Self {
        let mut app = App {
            items,
            selected: vec![false; items.len()],
            order: Vec::new(),
            state: ListState::default(),
            sort: Sort::Size,
            filtering: false,
            query: String::new(),
        };
        app.recompute();
        app
    }

    /// Rebuild `order` from the current filter query and sort mode.
    fn recompute(&mut self) {
        let q = self.query.to_lowercase();
        let mut idx: Vec<usize> = (0..self.items.len())
            .filter(|&i| {
                if q.is_empty() {
                    return true;
                }
                let it = &self.items[i];
                it.label.to_lowercase().contains(&q)
                    || it.ecosystem.to_lowercase().contains(&q)
                    || it.path.display().to_string().to_lowercase().contains(&q)
            })
            .collect();
        match self.sort {
            Sort::Size => idx.sort_by_key(|&i| std::cmp::Reverse(self.items[i].size)),
            Sort::Name => idx.sort_by(|&a, &b| self.items[a].path.cmp(&self.items[b].path)),
            Sort::Age => idx.sort_by(|&a, &b| {
                match (self.items[a].modified, self.items[b].modified) {
                    (Some(x), Some(y)) => x.cmp(&y), // older (smaller time) first
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                }
            }),
        }
        self.order = idx;
        match self.state.selected() {
            Some(p) if p >= self.order.len() => self.state.select(self.order.len().checked_sub(1)),
            None if !self.order.is_empty() => self.state.select(Some(0)),
            _ => {}
        }
    }

    fn move_cursor(&mut self, delta: isize) {
        if self.order.is_empty() {
            return;
        }
        let cur = self.state.selected().unwrap_or(0) as isize;
        let next = (cur + delta).rem_euclid(self.order.len() as isize) as usize;
        self.state.select(Some(next));
    }

    fn toggle_current(&mut self) {
        if let Some(&i) = self.state.selected().and_then(|p| self.order.get(p)) {
            self.selected[i] = !self.selected[i];
        }
    }

    fn toggle_all_visible(&mut self) {
        let all_on = !self.order.is_empty() && self.order.iter().all(|&i| self.selected[i]);
        for &i in &self.order {
            self.selected[i] = !all_on;
        }
    }

    fn picks(&self) -> Vec<usize> {
        self.selected
            .iter()
            .enumerate()
            .filter_map(|(i, &on)| on.then_some(i))
            .collect()
    }
}

/// Show an interactive multi-select picker over `items`.
/// Returns the indices the user chose to reclaim (empty = cancelled).
pub fn select(items: &[Reclaimable]) -> Result<Vec<usize>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;

    let result = run_loop(&mut terminal, items);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_loop<B: Backend>(terminal: &mut Terminal<B>, items: &[Reclaimable]) -> Result<Vec<usize>> {
    let mut app = App::new(items);
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if app.filtering {
            match key.code {
                KeyCode::Esc => {
                    app.query.clear();
                    app.filtering = false;
                    app.recompute();
                }
                KeyCode::Enter => app.filtering = false,
                KeyCode::Backspace => {
                    app.query.pop();
                    app.recompute();
                }
                KeyCode::Char(c) => {
                    app.query.push(c);
                    app.recompute();
                }
                _ => {}
            }
            continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Vec::new()),
            KeyCode::Char('/') => app.filtering = true,
            KeyCode::Char('s') => {
                app.sort = app.sort.next();
                app.recompute();
            }
            KeyCode::Down | KeyCode::Char('j') => app.move_cursor(1),
            KeyCode::Up | KeyCode::Char('k') => app.move_cursor(-1),
            KeyCode::Char(' ') => app.toggle_current(),
            KeyCode::Char('a') => app.toggle_all_visible(),
            KeyCode::Enter => return Ok(app.picks()),
            _ => {}
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.area());

    let rows: Vec<ListItem> = app
        .order
        .iter()
        .map(|&i| {
            let it = &app.items[i];
            let mark = if app.selected[i] { "[x]" } else { "[ ]" };
            ListItem::new(format!(
                "{mark} {:>10}  {:<13} {}",
                report::human(it.size),
                it.label,
                it.path.display()
            ))
        })
        .collect();

    let title = format!(
        " chaff — sort:{} · / filter · space · a all · enter reclaim · q quit ",
        app.sort.label()
    );
    let list = List::new(rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_symbol("➤ ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, chunks[0], &mut app.state);

    let total: u64 = app
        .selected
        .iter()
        .enumerate()
        .filter_map(|(i, &on)| on.then_some(app.items[i].size))
        .sum();
    let count = app.selected.iter().filter(|&&s| s).count();
    let footer_text = if app.filtering {
        format!(" filter: {}▌  ({} shown) ", app.query, app.order.len())
    } else {
        format!(
            " selected {count} item(s) — {} to reclaim (→ trash) ",
            report::human(total)
        )
    };
    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    fn it(
        label: &'static str,
        eco: &'static str,
        size: u64,
        age_days: u64,
        path: &str,
    ) -> Reclaimable {
        Reclaimable {
            path: PathBuf::from(path),
            ecosystem: eco,
            label,
            size,
            modified: Some(SystemTime::now() - Duration::from_secs(age_days * 86_400)),
        }
    }

    fn sample() -> Vec<Reclaimable> {
        vec![
            it("node_modules", "node", 100, 1, "/a/app/node_modules"),
            it("target", "rust", 300, 365, "/a/svc/target"),
            it(".venv", "python", 50, 30, "/a/ml/.venv"),
        ]
    }

    #[test]
    fn sorts_by_size_desc_by_default() {
        let items = sample();
        let app = App::new(&items);
        assert_eq!(app.order, vec![1, 0, 2]);
    }

    #[test]
    fn filter_narrows_order() {
        let items = sample();
        let mut app = App::new(&items);
        app.query = "node".into();
        app.recompute();
        assert_eq!(app.order, vec![0]);
    }

    #[test]
    fn age_sort_puts_oldest_first() {
        let items = sample();
        let mut app = App::new(&items);
        app.sort = Sort::Age;
        app.recompute();
        assert_eq!(app.order.first(), Some(&1));
    }

    #[test]
    fn toggle_all_visible_respects_filter() {
        let items = sample();
        let mut app = App::new(&items);
        app.query = "node".into();
        app.recompute();
        app.toggle_all_visible();
        assert_eq!(app.picks(), vec![0]);
    }
}
