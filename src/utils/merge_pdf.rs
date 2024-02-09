use std::collections::BTreeMap;

use lopdf::{Bookmark, Document, Object, ObjectId};
use web_sys::{console, js_sys::Array, js_sys::Uint8Array, Blob, File, FileList};

async fn read_file_content(file: File) -> Vec<u8> {
    let file_0 = file.array_buffer();
    let fut = wasm_bindgen_futures::JsFuture::from(file_0).await.unwrap();

    let uint8_array = Uint8Array::new(&fut);

    let vec = uint8_array.to_vec();
    vec
}

pub fn vec_to_uint8_array(data: Vec<u8>) -> Uint8Array {
    let uint8_array = Uint8Array::new_with_length(data.len() as u32);
    uint8_array.copy_from(data.as_slice());
    uint8_array
}

pub async fn merge_pdf_files(docs: FileList) -> Option<Blob> {
    // Define a starting max_id (will be used as start index for object_ids)
    let mut max_id = 1;
    let mut pagenum = 1;

    // Collect all Documents Objects grouped by a map
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.5");

    let docs_index = docs.length();

    for index in 0..docs_index {
        // Get the document as blob

        if let Some(docu) = docs.item(index) {
            let u8_array = read_file_content(docu).await;

            let mut doc = match Document::load_mem(&u8_array) {
                Ok(doc) => doc,
                Err(_) => {
                    console::log_1(&"Buffer Error".into());
                    return None;
                }
            };
            let mut first = false;
            doc.renumber_objects_with(max_id);

            max_id = doc.max_id + 1;

            documents_pages.extend(
                doc.get_pages()
                    .into_iter()
                    .map(|(_, object_id)| {
                        if !first {
                            let bookmark = Bookmark::new(
                                String::from(format!("Page_{}", pagenum)),
                                [0.0, 0.0, 1.0],
                                0,
                                object_id,
                            );
                            document.add_bookmark(bookmark, None);
                            first = true;
                            pagenum += 1;
                        }

                        (object_id, doc.get_object(object_id).unwrap().to_owned())
                    })
                    .collect::<BTreeMap<ObjectId, Object>>(),
            );
            documents_objects.extend(doc.objects);
        }
    }

    // Catalog and Pages are mandatory
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    // Process all objects except "Page" type
    for (object_id, object) in documents_objects.iter() {
        // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
        // All other objects should be collected and inserted into the main Document
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                // Collect a first "Catalog" object and use it for the future "Pages"
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        *object_id
                    },
                    object.clone(),
                ));
            }
            "Pages" => {
                // Collect and update a first "Pages" object and use it for the future "Catalog"
                // We have also to merge all dictionaries of the old and the new "Pages" object
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }

                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            *object_id
                        },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            "Page" => {}     // Ignored, processed later and separately
            "Outlines" => {} // Ignored, not supported yet
            "Outline" => {}  // Ignored, not supported yet
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    // If no "Pages" object found abort
    if pages_object.is_none() {
        console::log_1(&"Pages root not found.".into());

        return None;
    }

    // Iterate over all "Page" objects and collect into the parent "Pages" created before
    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);

            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    // If no "Catalog" found abort
    if catalog_object.is_none() {
        console::log_1(&"Catalog root not found.".into());

        return None;
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    // Build a new "Pages" with updated fields
    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();

        // Set new pages count
        dictionary.set("Count", documents_pages.len() as u32);

        // Set new "Kids" list (collected from documents pages) for "Pages"
        dictionary.set(
            "Kids",
            documents_pages
                .into_iter()
                .map(|(object_id, _)| Object::Reference(object_id))
                .collect::<Vec<_>>(),
        );

        document
            .objects
            .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build a new "Catalog" with updated fields
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

        document
            .objects
            .insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_object.0);

    // Update the max internal ID as wasn't updated before due to direct objects insertion
    document.max_id = document.objects.len() as u32;

    // Reorder all new Document objects
    document.renumber_objects();

    //Set any Bookmarks to the First child if they are not set to a page
    document.adjust_zero_pages();

    //Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
    if let Some(n) = document.build_outline() {
        if let Ok(x) = document.get_object_mut(catalog_object.0) {
            if let Object::Dictionary(ref mut dict) = x {
                dict.set("Outlines", Object::Reference(n));
            }
        }
    }

    document.compress();

    let mut doc_bits: Vec<u8> = vec![];

    match document.save_to(&mut doc_bits) {
        Ok(_) => {}
        Err(e) => {
            console::log_1(&format!("Error saving PDF: {}", e).into());

            return None;
        }
    };
    let uint8_array = vec_to_uint8_array(doc_bits);

    let array_buffer = Array::new();
    array_buffer.push(&uint8_array.buffer());

    let blob = match Blob::new_with_u8_array_sequence_and_options(
        &array_buffer.into(),
        web_sys::BlobPropertyBag::new().type_("application/pdf"),
    ) {
        Ok(blob) => blob,
        Err(_) => {
            console::log_1(&format!("Error saving PDF1").into());

            return None;
        }
    };

    Some(blob)
}
