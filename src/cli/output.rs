use colored::Colorize;
use prettytable::{Cell, Row, Table};
use serde_json::Value;

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an error message
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

/// Print a warning message
pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

/// Print an info message
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print a step message
pub fn step(msg: &str) {
    println!("{} {}", "→".cyan().bold(), msg);
}

/// Print a header
pub fn header(msg: &str) {
    println!("\n{}", msg.bold().underline());
}

/// Print key-value pair
pub fn key_value(key: &str, value: &str) {
    println!("  {}: {}", key.cyan(), value);
}

/// Print JSON output
pub fn json(data: &Value) {
    println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
}

/// Create a formatted table
pub fn create_table(headers: Vec<&str>) -> Table {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);

    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| Cell::new(h).style_spec("Fb"))
        .collect();
    table.add_row(Row::new(header_cells));

    table
}

/// Print a box with content
pub fn print_box(title: &str, content: Vec<(&str, &str)>) {
    let max_key_len = content.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let max_val_len = content.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
    let box_width = (max_key_len + max_val_len + 6).max(title.len() + 4);

    // Top border
    println!("┌{}┐", "─".repeat(box_width));

    // Title
    let title_padding = (box_width - title.len()) / 2;
    println!(
        "│{}{}{}│",
        " ".repeat(title_padding),
        title.bold(),
        " ".repeat(box_width - title.len() - title_padding)
    );

    // Separator
    println!("├{}┤", "─".repeat(box_width));

    // Content
    for (key, value) in content {
        let padding = max_key_len - key.len();
        println!(
            "│ {}{}: {}{}│",
            key.cyan(),
            " ".repeat(padding),
            value,
            " ".repeat(box_width - key.len() - value.len() - padding - 3)
        );
    }

    // Bottom border
    println!("└{}┘", "─".repeat(box_width));
}

/// Print a progress message
pub fn progress(msg: &str) {
    print!("{} {}...", "●".cyan(), msg);
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

/// Complete a progress message
pub fn progress_done() {
    println!(" {}", "done".green());
}
