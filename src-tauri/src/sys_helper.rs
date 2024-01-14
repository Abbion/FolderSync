use std::time::SystemTime;
use std::fs;

pub fn get_file_names_from_folder(path: & String) -> Vec<String> {
    let mut file_list = Vec::new();

    let itr = fs::read_dir(path);

    match itr {
        Ok(read_dir) => {
            for entry in read_dir {
                match entry {
                    Ok(dir_entry) => {
                        let file_path = dir_entry.path();
                        if file_path.is_file() {
                            if let Some(name) = file_path.file_name().and_then(|s| s.to_str()) {
                                file_list.push(name.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error while reading folder entry: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error while reading folder files: {}", e);
        }
    }

    return file_list
}

pub fn get_file_modification_date(file_path: &String) -> Option<SystemTime> {
    let metadata = fs::metadata(file_path);

    match metadata {
        Ok(data) => {
            match data.modified() {
                Ok(modification_date) => {
                    return Some(modification_date);
                },
                Err(e) => {
                    println!("Error while reading file modification date: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Error while reading file metadata: {}", e);
        }
    }

    None
}