use std::cmp::min;
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, Write};
use zip::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const MAX_PART_SIZE_BYTES: u64 = 100 * 1000 * 1000; // 100 MB

use crate::cryptography;

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

pub(crate) fn build_zip(files: &Vec<(String, String)>) -> (Vec<u8>, cryptography::FileMetadata) {
    let mut zip_bytes = Vec::new();
    let cursor = Cursor::new(&mut zip_bytes);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(None);

    for (file_name, content) in files {
        zip.start_file(&file_name, options).unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }

    zip.finish().unwrap();

    let meta = cryptography::get_metadata(&zip_bytes);
    (zip_bytes, meta)
}

pub fn calculate_batch_part_quantity(zip_size_bytes: u64) -> usize {
    if zip_size_bytes <= MAX_PART_SIZE_BYTES {
        return 1;
    }

    ((zip_size_bytes as f64) / (MAX_PART_SIZE_BYTES as f64)).ceil() as usize
}

pub fn split_bytes(input: &[u8], part_count: usize) -> Vec<Vec<u8>> {
    if part_count < 1 {
        return Vec::new();
    }

    let part_size = ((input.len() as f64) / (part_count as f64)).ceil() as usize;
    let mut result = Vec::with_capacity(part_count);

    for i in 0..part_count {
        let start = i * part_size;
        if start >= input.len() {
            break;
        }

        let end = min(start + part_size, input.len());
        result.push(input[start..end].to_vec());
    }

    result
}
