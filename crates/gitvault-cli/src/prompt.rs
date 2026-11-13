//! Small numbered menus on the terminal for the questions `gv trust` asks.
//!
//! Questions go to stderr so stdout stays free for results.

use std::io::{self, BufRead, Write};

use anyhow::{bail, Result};

fn ask(question: &str) -> Result<String> {
    eprint!("{question}");
    io::stderr().flush()?;
    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line)? == 0 {
        bail!("no answer given (the input ended)");
    }
    Ok(line.trim().to_owned())
}

fn print_options(title: &str, options: &[String]) {
    eprintln!("\n{title}");
    for (i, option) in options.iter().enumerate() {
        eprintln!("  {}) {option}", i + 1);
    }
}

/// Picks one option by number; Enter takes `default` (an index into `options`).
pub fn choose_one(title: &str, options: &[String], default: usize) -> Result<usize> {
    print_options(title, options);
    loop {
        let answer = ask(&format!("Choose 1-{} [{}]: ", options.len(), default + 1))?;
        if answer.is_empty() {
            return Ok(default);
        }
        match answer.parse::<usize>() {
            Ok(n) if (1..=options.len()).contains(&n) => return Ok(n - 1),
            _ => eprintln!("Type a number from 1 to {}.", options.len()),
        }
    }
}

/// Picks several options; Enter or `a` takes all of them.
pub fn choose_many(title: &str, options: &[String]) -> Result<Vec<usize>> {
    print_options(title, options);
    loop {
        let answer = ask("Numbers separated by commas, Enter for all: ")?;
        match parse_many(&answer, options.len()) {
            Some(picked) => return Ok(picked),
            None => eprintln!("Type numbers from 1 to {}, e.g. 1,3.", options.len()),
        }
    }
}

/// A yes/no question where Enter means yes.
pub fn confirm(question: &str) -> Result<bool> {
    let answer = ask(&format!("{question} [Y/n]: "))?.to_lowercase();
    Ok(answer.is_empty() || answer == "y" || answer == "yes")
}

/// Turns "1,3" / "1 3" / "" / "all" into zero-based indexes, or `None` when it is not valid.
fn parse_many(answer: &str, count: usize) -> Option<Vec<usize>> {
    let answer = answer.trim().to_lowercase();
    if answer.is_empty() || answer == "a" || answer == "all" {
        return Some((0..count).collect());
    }
    let mut picked = Vec::new();
    for part in answer.split([',', ' ']).filter(|p| !p.is_empty()) {
        let n: usize = part.parse().ok()?;
        if !(1..=count).contains(&n) {
            return None;
        }
        picked.push(n - 1);
    }
    picked.sort_unstable();
    picked.dedup();
    Some(picked)
}

#[cfg(test)]
mod tests {
    use super::parse_many;

    #[test]
    fn empty_and_all_take_everything() {
        assert_eq!(parse_many("", 3), Some(vec![0, 1, 2]));
        assert_eq!(parse_many("All", 2), Some(vec![0, 1]));
    }

    #[test]
    fn numbers_are_one_based_sorted_and_deduplicated() {
        assert_eq!(parse_many("3, 1 3", 3), Some(vec![0, 2]));
    }

    #[test]
    fn rejects_out_of_range_and_words() {
        assert_eq!(parse_many("4", 3), None);
        assert_eq!(parse_many("0", 3), None);
        assert_eq!(parse_many("dev", 3), None);
    }
}
