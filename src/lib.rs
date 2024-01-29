use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;
use web_sys::Blob;

use web_sys::FileList;

mod utils;

#[wasm_bindgen]
pub fn add(n1: i32, n2: i32) -> i32 {
    n1 + n2
}

#[wasm_bindgen]
pub fn print_file() {
    console::log_1(&"Hello world".into());
}

#[wasm_bindgen]
pub fn list_filename(files: FileList) {
    let files_index = files.length();
    for index in 0..files_index {
        if let Some(file) = files.item(index) {
            console::log_1(&file.name().into());
        }
    }
}

#[wasm_bindgen]
pub async fn join_files(files: FileList) -> Option<Blob> {
    utils::merge_pdf::merge_pdf_files(files).await
}
