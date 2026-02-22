use colored::*;
use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io::{stdout, Write};

pub fn clear_screen() {
    let _ = execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0));
}

pub fn interactive_menu(title: &str, items: &[(&str, &str)]) -> Option<char> {
    let n = items.len();

    let draw = |cur: usize| {
        let _ = execute!(stdout(), MoveTo(0, 5), Clear(ClearType::FromCursorDown));
        println!("{}", title.bold().cyan());
        for (i, (key, label)) in items.iter().enumerate() {
            if i == cur {
                println!("  {} {}", format!("[{}]", key).cyan(), label.bold());
            } else {
                println!("  {} {}", format!("[{}]", key).cyan(), label);
            }
        }
        println!();
        let _ = stdout().flush();
    };

    let mut cursor = 0usize;
    draw(cursor);

    let _ = enable_raw_mode();
    let result = loop {
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Up => {
                    cursor = if cursor == 0 { n - 1 } else { cursor - 1 };
                    let _ = disable_raw_mode();
                    draw(cursor);
                    let _ = enable_raw_mode();
                }
                KeyCode::Down => {
                    cursor = (cursor + 1) % n;
                    let _ = disable_raw_mode();
                    draw(cursor);
                    let _ = enable_raw_mode();
                }
                KeyCode::Enter => {
                    let _ = disable_raw_mode();
                    break Some(items[cursor].0.chars().next().unwrap());
                }
                KeyCode::Char(c) => {
                    let c = c.to_ascii_lowercase();
                    if items.iter().any(|(k, _)| k.chars().next() == Some(c)) {
                        let _ = disable_raw_mode();
                        break Some(c);
                    }
                }
                KeyCode::Esc => {
                    let _ = disable_raw_mode();
                    break None;
                }
                _ => {}
            }
        }
    };
    result
}

pub fn pause() {
    println!("{}", "Press ENTER to continue...".bright_black());
    let _ = std::io::stdin().read_line(&mut String::new());
}
