# exif-rewriter-rs

A Rust CLI tool to rename image files (JPG, JPEG) based on the DateTimeOriginal EXIF metadata. Renamed files will follow the format:

```
YYYYMMDD_HHMMSS.jpg
```

## Features

Scans a folder recursively for image files

Reads DateTimeOriginal EXIF metadata

Renames files using the EXIF timestamp

Skips files without proper EXIF data

## Usage

```
cargo run --release /path/to/folder