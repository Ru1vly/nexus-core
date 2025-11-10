use crate::cli::errors::{CliError, CliResult};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// Check if the daemon is running
pub fn is_running(pid_file: &Path) -> bool {
    if !pid_file.exists() {
        return false;
    }

    // Read PID from file
    let pid_str = match fs::read_to_string(pid_file) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return false,
    };

    let pid: i32 = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Check if process with PID is running
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Send signal 0 to check if process exists
        let result = unsafe { libc::kill(pid, 0) };
        if result == 0 {
            return true;
        }
        // Process doesn't exist, clean up PID file
        fs::remove_file(pid_file).ok();
        false
    }

    #[cfg(windows)]
    {
        // On Windows, try to open the process
        use std::os::windows::io::AsRawHandle;
        use std::ptr;
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid as u32);
            if handle != ptr::null_mut() {
                winapi::um::handleapi::CloseHandle(handle);
                return true;
            }
        }
        // Process doesn't exist, clean up PID file
        fs::remove_file(pid_file).ok();
        false
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Fallback: assume running if PID file exists
        true
    }
}

/// Get the PID from the PID file
pub fn get_pid(pid_file: &Path) -> CliResult<i32> {
    if !pid_file.exists() {
        return Err(CliError::DaemonError("Daemon is not running".to_string()));
    }

    let pid_str = fs::read_to_string(pid_file)?;
    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|_| CliError::DaemonError("Invalid PID file".to_string()))?;

    Ok(pid)
}

/// Write PID to file
pub fn write_pid(pid_file: &Path, pid: i32) -> CliResult<()> {
    // Create parent directory if needed
    if let Some(parent) = pid_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(pid_file, pid.to_string())?;
    Ok(())
}

/// Remove PID file
pub fn remove_pid_file(pid_file: &Path) -> CliResult<()> {
    if pid_file.exists() {
        fs::remove_file(pid_file)?;
    }
    Ok(())
}

/// Stop the daemon process
pub fn stop_daemon(pid_file: &Path) -> CliResult<()> {
    if !is_running(pid_file) {
        return Err(CliError::DaemonError("Daemon is not running".to_string()));
    }

    let pid = get_pid(pid_file)?;

    #[cfg(unix)]
    {
        // Send SIGTERM to gracefully stop the process
        unsafe {
            if libc::kill(pid, libc::SIGTERM) != 0 {
                return Err(CliError::DaemonError(format!(
                    "Failed to stop daemon (PID: {})",
                    pid
                )));
            }
        }
    }

    #[cfg(windows)]
    {
        // On Windows, use taskkill
        let output = Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .output()?;

        if !output.status.success() {
            return Err(CliError::DaemonError(format!(
                "Failed to stop daemon (PID: {})",
                pid
            )));
        }
    }

    // Wait a moment for the process to stop
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Remove PID file
    remove_pid_file(pid_file)?;

    Ok(())
}

/// Start the daemon in the background
#[cfg(unix)]
pub fn daemonize(pid_file: &Path) -> CliResult<()> {
    use daemonize::Daemonize;
    use std::fs::File;

    let log_file = pid_file
        .parent()
        .unwrap_or(Path::new("/tmp"))
        .join("nexus-daemon.log");

    let stdout = File::create(&log_file)
        .map_err(|e| CliError::DaemonError(format!("Failed to create log file: {}", e)))?;
    let stderr = File::create(&log_file)
        .map_err(|e| CliError::DaemonError(format!("Failed to create log file: {}", e)))?;

    let daemonize = Daemonize::new()
        .pid_file(pid_file)
        .working_directory(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
        .stdout(stdout)
        .stderr(stderr);

    daemonize
        .start()
        .map_err(|e| CliError::DaemonError(format!("Failed to daemonize: {}", e)))?;

    Ok(())
}

/// Start the daemon in the background (Windows fallback)
#[cfg(windows)]
pub fn daemonize(pid_file: &Path) -> CliResult<()> {
    // On Windows, we'll just spawn a detached process
    // The actual implementation would use Windows services, but for simplicity
    // we'll just note that full daemon support requires more work on Windows
    Err(CliError::DaemonError(
        "Daemon mode is not fully supported on Windows. Run in foreground mode.".to_string(),
    ))
}

/// Get daemon uptime in seconds
pub fn get_uptime(pid_file: &Path) -> CliResult<u64> {
    if !is_running(pid_file) {
        return Err(CliError::DaemonError("Daemon is not running".to_string()));
    }

    // Get file modification time as proxy for start time
    let metadata = fs::metadata(pid_file)?;
    let modified = metadata.modified()?;
    let now = std::time::SystemTime::now();

    let duration = now
        .duration_since(modified)
        .map_err(|_| CliError::DaemonError("Failed to calculate uptime".to_string()))?;

    Ok(duration.as_secs())
}

/// Format uptime duration
pub fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

use std::path::PathBuf;
