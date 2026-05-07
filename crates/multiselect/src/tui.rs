use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};

use crate::model::{RenderRow, Tree};

struct TerminalGuard {
    tty: File,
}

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        let mut tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?;
        terminal::enable_raw_mode()?;
        execute!(tty, EnterAlternateScreen, cursor::Hide)?;
        Ok(Self { tty })
    }

    fn writer(&mut self) -> &mut File {
        &mut self.tty
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(self.tty, cursor::Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

pub(crate) fn run(prompt: &str, mut tree: Tree) -> Result<Option<Vec<String>>, String> {
    let mut guard = TerminalGuard::new().map_err(|e| e.to_string())?;
    let mut cursor_idx: usize = 0;

    loop {
        let order = tree.render_order();
        if !order.is_empty() && cursor_idx >= order.len() {
            cursor_idx = order.len() - 1;
        }
        draw(guard.writer(), prompt, &tree, &order, cursor_idx).map_err(|e| e.to_string())?;

        let evt = event::read().map_err(|e| e.to_string())?;
        let Event::Key(KeyEvent { code, modifiers, kind, .. }) = evt else {
            continue;
        };
        if kind != KeyEventKind::Press {
            continue;
        }

        match (code, modifiers) {
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                if cursor_idx > 0 {
                    cursor_idx -= 1;
                }
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                if cursor_idx + 1 < order.len() {
                    cursor_idx += 1;
                }
            }
            (KeyCode::Char(' '), _) => {
                if let Some(row) = order.get(cursor_idx) {
                    tree.toggle(row.item_idx);
                }
            }
            (KeyCode::Enter, _) => return Ok(Some(tree.selected_leaves())),
            (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::NONE) => return Ok(None),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(None),
            _ => {}
        }
    }
}

fn draw(
    out: &mut impl Write,
    prompt: &str,
    tree: &Tree,
    order: &[RenderRow],
    cursor_idx: usize,
) -> io::Result<()> {
    queue!(out, Clear(ClearType::All), cursor::MoveTo(0, 0), Print(prompt))?;

    let mut row: u16 = 1;
    let mut printed_any = false;
    for (i, r) in order.iter().enumerate() {
        if r.has_children && printed_any {
            row += 1;
        }
        let item = &tree.items[r.item_idx];
        let indent = " ".repeat(r.depth * 2);
        let glyph = if item.selected { "[\u{2713}]" } else { "[ ]" };
        let line = format!("{}{} {}", indent, glyph, item.label);
        queue!(out, cursor::MoveTo(0, row))?;
        if i == cursor_idx {
            queue!(
                out,
                SetAttribute(Attribute::Reverse),
                Print(&line),
                SetAttribute(Attribute::Reset)
            )?;
        } else {
            queue!(out, Print(&line))?;
        }
        row += 1;
        printed_any = true;
    }

    row += 1;
    queue!(
        out,
        cursor::MoveTo(0, row),
        Print("space toggle \u{00B7} enter confirm \u{00B7} esc cancel"),
    )?;
    out.flush()?;
    Ok(())
}
