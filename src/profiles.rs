use std::path::Path;

pub fn list() -> Vec<String> {
    let mut profiles = Vec::new();

    if let Ok(entries) = std::fs::read_dir(".mcvcli.profiles") {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir()
                && Path::new(&path).join(".mcvcli.json").exists()
                && let Some(name) = path.file_name().and_then(|name| name.to_str())
            {
                profiles.push(name.to_string());
            }
        }
    } else {
        return profiles;
    }

    profiles.sort();

    profiles
}
