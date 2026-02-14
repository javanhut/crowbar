use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub categories: Vec<String>,
    pub no_display: bool,
}

pub fn load_desktop_entries() -> Vec<DesktopEntry> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dirs = [
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from(format!("{home}/.local/share/applications")),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        PathBuf::from(format!("{home}/.local/share/flatpak/exports/share/applications")),
    ];

    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for dir in &dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(read_dir) = std::fs::read_dir(dir) else {
            continue;
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }

            if let Some(desktop_entry) = parse_desktop_file(&path) {
                if desktop_entry.no_display {
                    continue;
                }
                if seen.insert(desktop_entry.name.clone()) {
                    entries.push(desktop_entry);
                }
            }
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    entries
}

fn parse_desktop_file(path: &PathBuf) -> Option<DesktopEntry> {
    let content = std::fs::read_to_string(path).ok()?;

    let mut in_desktop_entry = false;
    let mut fields = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            fields.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let name = fields.get("Name")?.clone();
    let exec = fields.get("Exec")?.clone();

    // Skip entries of type Link or Directory
    if let Some(entry_type) = fields.get("Type") {
        if entry_type != "Application" {
            return None;
        }
    }

    let no_display = fields.get("NoDisplay")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let hidden = fields.get("Hidden")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if hidden {
        return None;
    }

    let icon = fields.get("Icon").cloned();
    let comment = fields.get("Comment").cloned();
    let categories = fields.get("Categories")
        .map(|c| c.split(';').filter(|s| !s.is_empty()).map(String::from).collect())
        .unwrap_or_default();

    Some(DesktopEntry {
        name,
        exec,
        icon,
        comment,
        categories,
        no_display,
    })
}

pub fn search_entries<'a>(entries: &'a [DesktopEntry], query: &str) -> Vec<&'a DesktopEntry> {
    if query.is_empty() {
        return entries.iter().take(20).collect();
    }

    let query_lower = query.to_lowercase();

    let mut results: Vec<(usize, &DesktopEntry)> = entries
        .iter()
        .filter_map(|entry| {
            let name_lower = entry.name.to_lowercase();
            let comment_lower = entry.comment.as_deref().unwrap_or("").to_lowercase();
            let cats_lower: String = entry.categories.join(" ").to_lowercase();

            // Priority: name starts with > name contains > comment contains > categories contains
            if name_lower.starts_with(&query_lower) {
                Some((0, entry))
            } else if name_lower.contains(&query_lower) {
                Some((1, entry))
            } else if comment_lower.contains(&query_lower) {
                Some((2, entry))
            } else if cats_lower.contains(&query_lower) {
                Some((3, entry))
            } else {
                None
            }
        })
        .collect();

    results.sort_by_key(|(priority, _)| *priority);
    results.into_iter().map(|(_, entry)| entry).take(10).collect()
}

pub fn launch_app(entry: &DesktopEntry) {
    let exec = clean_exec(&entry.exec);
    if exec.is_empty() {
        return;
    }

    // Split into command and args
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd = parts[0];
    let args = &parts[1..];

    let _ = Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn clean_exec(exec: &str) -> String {
    // Remove field codes: %f, %F, %u, %U, %d, %D, %n, %N, %i, %c, %k, %v, %m
    let mut result = exec.to_string();
    for code in &["%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m"] {
        result = result.replace(code, "");
    }
    // Remove "env " prefix variations
    if result.starts_with("env ") {
        // Find the actual command after env vars
        let parts: Vec<&str> = result.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                continue; // skip "env"
            }
            if !part.contains('=') {
                result = parts[i..].join(" ");
                break;
            }
        }
    }
    result.trim().to_string()
}
