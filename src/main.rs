use std::fmt::Display;
use std::fs::{File, read_dir};
use std::io::BufReader;
use std::path::Path;
use std::{env, path::PathBuf};

use chrono::{DateTime, TimeZone, Utc};
use little_exif::exif_tag::ExifTag::DateTimeOriginal;
use little_exif::metadata::Metadata;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json;

#[derive(Debug)]
enum Error {
    SerdeError(serde_json::Error),
    IoError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SerdeError(serde_error) => write!(f, "{}", serde_error),
            Error::IoError(io_error) => write!(f, "{}", io_error),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

#[derive(Debug, Deserialize)]
struct AlbumPhoto {
    uri: String,
    creation_timestamp: i64,
    media_metadata: MediaMetadata,
    title: String,
    backup_uri: String,
}

#[derive(Debug, Deserialize)]
struct MiscPhoto {
    uri: String,
    creation_timestamp: i64,
    media_metadata: MediaMetadata,
    backup_uri: String,
}

#[derive(Debug, Deserialize)]
struct MediaMetadata {
    photo_metadata: PhotoMetadata,
}

#[derive(Debug, Deserialize)]
struct PhotoMetadata {
    exif_data: Vec<ExifData>,
}

#[derive(Debug, Deserialize)]
struct ExifData {
    upload_ip: String,
    #[serde(default)]
    taken_timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct Album {
    name: String,
    photos: Vec<AlbumPhoto>,
    last_modified_timestamp: i64,
    description: String,
}

#[derive(Debug, Deserialize)]
struct Misc {
    #[serde(rename = "other_photos_v2")]
    photos: Vec<MiscPhoto>,
}

impl Misc {
    fn add_metadata(&self, dir: &Path, metadata: &mut Metadata) {
        for photo in &self.photos {
            let taken = photo
                .media_metadata
                .photo_metadata
                .exif_data
                .get(0)
                .unwrap()
                .taken_timestamp;

            let creation = photo.creation_timestamp;

            let date = get_date(creation, taken).unwrap();
            let img = dir.with_file_name(&photo.uri);
            let ente_date = format!("{}", date.format("%Y:%m:%d %H:%M:%S"));

            metadata.set_tag(DateTimeOriginal(ente_date));
            metadata.write_to_file(&img).unwrap();
        }
    }
}
impl Album {
    fn add_metadata(&self, dir: &Path, metadata: &mut Metadata) {
        for photo in &self.photos {
            let taken = photo
                .media_metadata
                .photo_metadata
                .exif_data
                .get(0)
                .unwrap()
                .taken_timestamp;

            let creation = photo.creation_timestamp;

            let date = get_date(creation, taken).unwrap();
            let img = dir.with_file_name(&photo.uri);
            let ente_date = format!("{}", date.format("%Y:%m:%d %H:%M:%S"));

            metadata.set_tag(DateTimeOriginal(ente_date));
            metadata.write_to_file(&img).unwrap();
        }
    }
}

fn parse_json<T: DeserializeOwned>(file: &Path) -> Result<T, Error> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    let d: T = serde_json::from_reader(reader)?;

    Ok(d)
}

fn get_date(creation: i64, taken: i64) -> Result<DateTime<Utc>, Error> {
    let date = if taken == 0 {
        Utc.timestamp_opt(creation, 0).unwrap()
    } else {
        Utc.timestamp_opt(taken, 0).unwrap()
    };

    Ok(date)
}

fn get_misc_photos_data(dir: &Path) -> Result<Misc, Error> {
    let file = dir.with_file_name("your_facebook_activity/posts/your_uncategorized_photos.json");

    parse_json(&file)
}

fn get_album_files(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let album_dir = dir.with_file_name("your_facebook_activity/posts/album");
    let json = std::ffi::OsStr::new("json");

    let album_files = Vec::from_iter(
        read_dir(album_dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|e| e.extension() == Some(json)),
    );

    Ok(album_files)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = Path::new(&args[1]);
    let misc = get_misc_photos_data(dir).unwrap();
    let albums = get_album_files(dir).unwrap();

    // For writing image metadata
    let mut metadata = Metadata::new();

    misc.add_metadata(dir, &mut metadata);

    for file in albums {
        let tmp: Album = parse_json(&file).unwrap();
        tmp.add_metadata(dir, &mut metadata);
    }
}
