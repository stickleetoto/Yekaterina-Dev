use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::user_ops::{UserRegistry, UserSnapshot};

const KEEP_SNAPSHOTS: usize = 3;
const SNAPSHOT_PREFIX: &str = "snapshot-";
const SNAPSHOT_SUFFIX: &str = ".json";

pub fn default_store_dir() -> PathBuf {
    if let Some(p) = env::var_os("YEKATERINA_HOME") {
        return PathBuf::from(p).join("udo");
    }

    #[cfg(windows)]
    if let Some(p) = env::var_os("LOCALAPPDATA") {
        return PathBuf::from(p).join("Yekaterina").join("udo");
    }

    if let Some(p) = env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(p).join("yekaterina").join("udo");
    }
    if let Some(p) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
        return PathBuf::from(p).join(".yekaterina").join("udo");
    }
    PathBuf::from(".yekaterina").join("udo")
}

pub fn load(dir: &Path) -> Result<UserRegistry, &'static str> {
    if !dir.exists() { return Ok(UserRegistry::default()); }
    let mut snapshots = list_snapshots(dir)?;
    snapshots.sort_by_key(|(generation, _)| std::cmp::Reverse(*generation));

    for (_, path) in snapshots {
        let bytes = match fs::read(&path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let snapshot: UserSnapshot = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Ok(registry) = UserRegistry::from_snapshot(snapshot) {
            return Ok(registry);
        }
    }
    Ok(UserRegistry::default())
}

pub fn save(dir: &Path, registry: &UserRegistry) -> Result<(), &'static str> {
    fs::create_dir_all(dir).map_err(|_| "IO")?;
    let existing = list_snapshots(dir)?;
    let generation = existing.iter().map(|(g, _)| *g).max().unwrap_or(0).saturating_add(1);
    let final_name = format!("{SNAPSHOT_PREFIX}{generation:020}{SNAPSHOT_SUFFIX}");
    let temp_name = format!(".{final_name}.tmp-{}", std::process::id());
    let final_path = dir.join(final_name);
    let temp_path = dir.join(temp_name);

    let payload = serde_json::to_vec(&registry.snapshot()).map_err(|_| "IO")?;
    let mut file = File::create(&temp_path).map_err(|_| "IO")?;
    file.write_all(&payload).map_err(|_| "IO")?;
    file.flush().map_err(|_| "IO")?;
    file.sync_all().map_err(|_| "IO")?;
    drop(file);

    // Destination is generation-unique, so rename never needs to overwrite an
    // existing file. This preserves atomic publish semantics on Windows too.
    fs::rename(&temp_path, &final_path).map_err(|_| "IO")?;
    // The rename above is the commit point. Post-commit durability/retention
    // maintenance is best-effort so callers never roll back in-memory state
    // after a snapshot has already become authoritative on disk.
    let _ = sync_directory(dir);
    let _ = cleanup_old(dir);
    Ok(())
}

#[cfg(unix)]
fn sync_directory(dir: &Path) -> std::io::Result<()> {
    File::open(dir)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_dir: &Path) -> std::io::Result<()> { Ok(()) }

fn cleanup_old(dir: &Path) -> std::io::Result<()> {
    let mut snapshots = list_snapshots_io(dir)?;
    snapshots.sort_by_key(|(generation, _)| std::cmp::Reverse(*generation));
    for (_, path) in snapshots.into_iter().skip(KEEP_SNAPSHOTS) {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

fn list_snapshots(dir: &Path) -> Result<Vec<(u64, PathBuf)>, &'static str> {
    list_snapshots_io(dir).map_err(|_| "IO")
}

fn list_snapshots_io(dir: &Path) -> std::io::Result<Vec<(u64, PathBuf)>> {
    let mut out = Vec::new();
    if !dir.exists() { return Ok(out); }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() { continue; }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(body) = name.strip_prefix(SNAPSHOT_PREFIX).and_then(|s| s.strip_suffix(SNAPSHOT_SUFFIX)) else {
            continue;
        };
        if let Ok(generation) = body.parse::<u64>() {
            out.push((generation, entry.path()));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{env, time::{SystemTime, UNIX_EPOCH}};

    fn temp_dir() -> PathBuf {
        let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        env::temp_dir().join(format!("yekaterina-storage-test-{}-{n}", std::process::id()))
    }


    #[test]
    fn corrupt_latest_falls_back_to_previous_snapshot() {
        let dir = temp_dir();
        let mut registry = UserRegistry::default();
        registry.define_formula(&[json!({"op":"user.one","p":["x"],"expr":"x+1"})]).unwrap();
        save(&dir, &registry).unwrap();
        registry.define_formula(&[json!({"op":"user.two","p":["x"],"expr":"x+2"})]).unwrap();
        save(&dir, &registry).unwrap();

        let mut snapshots = list_snapshots(&dir).unwrap();
        snapshots.sort_by_key(|(g, _)| std::cmp::Reverse(*g));
        fs::write(&snapshots[0].1, b"not-json").unwrap();

        let loaded = load(&dir).unwrap();
        assert!(loaded.lookup("user.one").is_some());
        assert!(loaded.lookup("user.two").is_none());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn snapshot_round_trip() {
        let dir = temp_dir();
        let mut registry = UserRegistry::default();
        registry.define_formula(&[json!({"op":"user.energy","p":["m","v"],"expr":"0.5*m*v^2"})]).unwrap();
        save(&dir, &registry).unwrap();
        let loaded = load(&dir).unwrap();
        assert!(loaded.lookup("user.energy").is_some());
        let _ = fs::remove_dir_all(dir);
    }
}
