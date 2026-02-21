use std::collections::HashMap;
use std::io::{Read, Seek};
use zip::ZipArchive;

pub(crate) fn unzip<R: Read + Seek>(zip_stream: R) -> HashMap<String, String> {
    let mut archive = ZipArchive::new(zip_stream).unwrap();
    let mut files = HashMap::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();

        if entry.name().trim().is_empty() {
            continue;
        }

        let mut content = String::new();
        entry.read_to_string(&mut content).unwrap();

        files.insert(entry.name().to_string(), content);
    }

    files
}
