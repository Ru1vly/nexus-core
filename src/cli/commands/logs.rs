use crate::cli::config::Config;
use crate::cli::errors::{CliError, CliResult};
use crate::cli::output;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use tokio::time::{sleep, Duration};

pub async fn view(
    follow: bool,
    lines: usize,
    level: Option<&str>,
    config: &Config,
) -> CliResult<()> {
    let log_path = config.log_path();

    if !Path::new(&log_path).exists() {
        output::warning(&format!("Log file not found: {}", log_path));
        output::info("The log file will be created when the daemon starts");
        return Ok(());
    }

    if follow {
        // Follow mode - tail -f style
        tail_follow(&log_path, level).await
    } else {
        // Single read - show last N lines
        tail_lines(&log_path, lines, level)
    }
}

fn tail_lines(log_path: &str, lines: usize, level: Option<&str>) -> CliResult<()> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);

    let all_lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .collect();

    let start_idx = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };

    for line in &all_lines[start_idx..] {
        if should_show_line(line, level) {
            println!("{}", line);
        }
    }

    Ok(())
}

async fn tail_follow(log_path: &str, level: Option<&str>) -> CliResult<()> {
    let mut file = File::open(log_path)?;
    let mut reader = BufReader::new(file);

    // Seek to end
    reader.seek(SeekFrom::End(0))?;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // No new data, wait and try again
                sleep(Duration::from_millis(100)).await;

                // Re-open file in case it was rotated
                file = File::open(log_path)?;
                reader = BufReader::new(file);
            }
            Ok(_) => {
                if should_show_line(&line, level) {
                    print!("{}", line);
                }
            }
            Err(e) => {
                return Err(CliError::IoError(e));
            }
        }
    }
}

fn should_show_line(line: &str, level_filter: Option<&str>) -> bool {
    if let Some(level) = level_filter {
        let level_upper = level.to_uppercase();
        line.contains(&level_upper)
    } else {
        true
    }
}
