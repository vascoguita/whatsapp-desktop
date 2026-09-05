use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

pub const LOG_FILE_NAME: &str = "whatsapp-desktop";
const REPORT_EMAIL: &str = "whatsapp-desktop@guita.org";
// mailto: links are passed to the OS/mail client as a single URL, and many
// clients truncate or reject very long ones, so only the tail of the log is
// included rather than the whole (already size-capped) log file.
const MAX_LOG_CHARS_IN_REPORT: usize = 4000;

pub fn open_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("report") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "report", WebviewUrl::App("report.html".into()))
        .title("Report an Issue")
        .inner_size(480.0, 420.0)
        .resizable(false)
        .center()
        .build()?;
    window.remove_menu()?;

    Ok(())
}

/// Keeps only the last `max_chars` characters of `log`, returning the tail
/// alongside whether it had to be truncated to get there.
fn tail(log: &str, max_chars: usize) -> (String, bool) {
    let char_count = log.chars().count();
    let truncated = char_count > max_chars;
    let tail = log
        .chars()
        .skip(char_count.saturating_sub(max_chars))
        .collect();
    (tail, truncated)
}

fn report_body(description: &str, log: &str) -> String {
    let (log_tail, truncated) = tail(log, MAX_LOG_CHARS_IN_REPORT);

    let mut body = description.trim().to_string();
    body.push_str("\n\n--- Application log");
    if truncated {
        body.push_str(" (most recent lines only)");
    }
    body.push_str(" ---\n");
    body.push_str(if log.is_empty() {
        "(no log file found)"
    } else {
        &log_tail
    });
    body
}

#[tauri::command]
pub fn submit_report(app: AppHandle, description: String) -> Result<(), String> {
    let log_path = app
        .path()
        .app_log_dir()
        .map_err(|err| err.to_string())?
        .join(LOG_FILE_NAME)
        .with_extension("log");
    let log = std::fs::read_to_string(&log_path).unwrap_or_default();
    let body = report_body(&description, &log);

    let subject = format!(
        "WhatsApp Desktop v{} - Issue Report",
        env!("CARGO_PKG_VERSION")
    );
    let mailto = format!(
        "mailto:{to}?subject={subject}&body={body}",
        to = REPORT_EMAIL,
        subject = utf8_percent_encode(&subject, NON_ALPHANUMERIC),
        body = utf8_percent_encode(&body, NON_ALPHANUMERIC),
    );

    app.opener()
        .open_url(mailto, None::<&str>)
        .map_err(|err| err.to_string())?;

    if let Some(window) = app.get_webview_window("report") {
        let _ = window.close();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_keeps_whole_log_when_under_the_limit() {
        let (result, truncated) = tail("short log", 4000);
        assert_eq!(result, "short log");
        assert!(!truncated);
    }

    #[test]
    fn tail_keeps_only_the_most_recent_chars_when_over_the_limit() {
        let log = "0123456789";
        let (result, truncated) = tail(log, 4);
        assert_eq!(result, "6789");
        assert!(truncated);
    }

    #[test]
    fn report_body_notes_missing_log_file() {
        let body = report_body("it crashed", "");
        assert!(body.starts_with("it crashed"));
        assert!(body.contains("(no log file found)"));
    }

    #[test]
    fn report_body_flags_truncation() {
        let log = "x".repeat(MAX_LOG_CHARS_IN_REPORT + 1);
        let body = report_body("desc", &log);
        assert!(body.contains("(most recent lines only)"));
    }
}
