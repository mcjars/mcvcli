use std::path::Path;

pub fn list() -> Vec<String> {
    let mut profiles = Vec::new();
    let entries = std::fs::read_dir(".mcvcli.profiles").ok();

    if entries.is_none() {
        return profiles;
    }

    for entry in entries.unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() && Path::new(&path).join(".mcvcli.json").exists() {
            let name = path.file_name().unwrap().to_str().unwrap();
            profiles.push(name.to_string());
        }
    }

    return profiles;
}
