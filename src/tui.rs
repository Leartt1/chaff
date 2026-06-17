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
use std::io::stdout;

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
    let mut selected = vec![false; items.len()];
    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(0));
    }

    loop {
        terminal.draw(|f| ui(f, items, &selected, &mut state))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Vec::new()),
            KeyCode::Down | KeyCode::Char('j') => move_cursor(&mut state, items.len(), 1),
            KeyCode::Up | KeyCode::Char('k') => move_cursor(&mut state, items.len(), -1),
            KeyCode::Char(' ') => {
                if let Some(i) = state.selected() {
                    selected[i] = !selected[i];
                }
            }
            KeyCode::Char('a') => {
                let all_on = selected.iter().all(|&s| s);
                selected.iter_mut().for_each(|s| *s = !all_on);
            }
            KeyCode::Enter => {
                let picks = selected
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &on)| on.then_some(i))
                    .collect();
                return Ok(picks);
            }
            _ => {}
        }
    }
}

fn move_cursor(state: &mut ListState, len: usize, delta: isize) {
    if len == 0 {
        return;
    }
    let current = state.selected().unwrap_or(0) as isize;
    let next = (current + delta).rem_euclid(len as isize) as usize;
    state.select(Some(next));
}

fn ui(f: &mut Frame, items: &[Reclaimable], selected: &[bool], state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.area());

    let rows: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let mark = if selected[i] { "[x]" } else { "[ ]" };
            ListItem::new(format!(
                "{mark} {:>10}  {:<13} {}",
                report::human(it.size),
                it.label,
                it.path.display()
            ))
        })
        .collect();

    let list = List::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" chaff — ↑/↓ move · space toggle · a all · enter reclaim · q cancel "),
        )
        .highlight_symbol("➤ ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, chunks[0], state);

    let total: u64 = items
        .iter()
        .zip(selected)
        .filter_map(|(it, &on)| on.then_some(it.size))
        .sum();
    let count = selected.iter().filter(|&&s| s).count();
    let footer = Paragraph::new(format!(
        " selected {count} item(s) — {} to reclaim (→ trash) ",
        report::human(total)
    ))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[1]);
}
