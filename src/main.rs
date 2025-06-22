use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser;
use clap_derive::Parser;
use walkdir::WalkDir;
use exif::{In, Reader, Tag, Value};

/// CLI arguments
#[derive(Parser)]
#[command(author, version, about = "Rename images based on EXIF DateTimeOriginal", long_about = None)]
struct Opt {
    /// Directory containing images to rename
    #[arg(value_name = "FOLDER")]
    folder: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let dir = &opt.folder;

    if !dir.is_dir() {
        anyhow::bail!("Provided path is not a directory: {:?}", dir);
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_image(path) {
            match process_file(path) {
                Ok(Some(new_path)) => println!("Renamed: {:?} -> {:?}", path, new_path),
                Ok(None) => println!("No DateTimeOriginal found for {:?}, skipped", path),
                Err(e) => eprintln!("Error processing {:?}: {}", path, e),
            }
        }
    }

    Ok(())
}

/// Check if file has a JPG extension
fn is_image(path: &Path) -> bool {
    match path.extension().and_then(OsStr::to_str) {
        Some(ext) => {
            let ext_lower = ext.to_lowercase();
            ext_lower == "jpg" || ext_lower == "jpeg"
        }
        None => false,
    }
}

/// Process a single file: read EXIF, extract DateTimeOriginal, and rename.
fn process_file(path: &Path) -> anyhow::Result<Option<PathBuf>> {
    let file = fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exif_reader = Reader::new();
    let exif = exif_reader.read_from_container(&mut bufreader)?;

    if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        if let Value::Ascii(ref vec) = field.value {
            if let Some(datetime) = vec.get(0) {
                let datetime = std::str::from_utf8(datetime)?;
                // Expected format: "YYYY:MM:DD HH:MM:SS"
                let formatted = format_datetime(datetime)?;
                let ext = path.extension().and_then(OsStr::to_str).unwrap_or("jpg");
                let parent = path.parent().unwrap_or(Path::new("."));
                let new_name = format!("{}.{}", formatted, ext.to_lowercase());
                let new_path = parent.join(new_name);
                fs::rename(path, &new_path)?;
                return Ok(Some(new_path));
            }
        }
    }

    Ok(None)
}

/// Convert EXIF DateTimeOriginal string into "YYYYMMDD_HHMMSS"
fn format_datetime(exif_dt: &str) -> anyhow::Result<String> {
    // exif_dt is like "2024:01:01 19:39:23"
    let parts: Vec<&str> = exif_dt.split(' ').collect();
    if parts.len() != 2 {
        anyhow::bail!("Unexpected DateTime format: {}", exif_dt);
    }
    let date = parts[0].replace(':', ""); // YYYYMMDD
    let time = parts[1].replace(':', ""); // HHMMSS
    Ok(format!("{}_{}", date, time))
}
