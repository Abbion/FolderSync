// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Mutex, Arc};
use std::path::Path;
use std::thread;
use std::time::Duration;

use serde::Serialize;

#[derive(serde::Deserialize, Debug, Clone, Serialize)]
enum IntervalType {
    SECOND,
    MINUTE,
    HOUR
}

#[derive(serde::Deserialize, Debug, Clone, Serialize)]
struct SyncData {
    from_path: String,
    to_path: String,
    interval_value: i32,
    interval_type: IntervalType,
    enabled: bool
}

struct Database {
    sync_entries: Mutex<HashMap<i64, SyncData>>,
    next_id: Mutex<i64>,
    edited_id: Mutex<Option<i64>>
}

#[tauri::command]
fn add_sync(sync_data: SyncData, id: i64, database: tauri::State<Arc<Database>>) -> bool {
    println!("Add new sync: {:?}, id: {}", sync_data, id);

    let mut sync_entries = database.sync_entries.lock().unwrap();
    
    if (*sync_entries).contains_key(&id) {
        return false;
    }

    (*sync_entries).insert(id, sync_data);

    //Save data in the database and return its state

    return true;
}

#[tauri::command]
fn delete_sync(id: i64, database: tauri::State<Arc<Database>>) -> bool {
    println!("Delete sync:, id: {}", id);

    let mut sync_entries: std::sync::MutexGuard<'_, HashMap<i64, SyncData>> = database.sync_entries.lock().unwrap();
    
    if (*sync_entries).contains_key(&id) {
        (*sync_entries).remove(&id);
        return true;
    }

    return false;
}

#[tauri::command]
fn replace_sync(mut sync_data: SyncData, id: i64, database: tauri::State<Arc<Database>>) -> bool {
    println!("Replace sync: {:?}, id: {}", sync_data, id);
    
    let mut sync_entries: std::sync::MutexGuard<'_, HashMap<i64, SyncData>> = database.sync_entries.lock().unwrap();
    
    if (*sync_entries).contains_key(&id) {
        let old_enabled = (*sync_entries).get(&id).unwrap().enabled;
        (*sync_entries).remove(&id);

        sync_data.enabled = old_enabled;
        (*sync_entries).insert(id, sync_data);
        return true;
    }

    return false;
}

#[tauri::command]
fn get_sync(id: i64, database: tauri::State<Arc<Database>>) -> Option<SyncData> {
    let sync_entries: std::sync::MutexGuard<'_, HashMap<i64, SyncData>> = database.sync_entries.lock().unwrap();

    match (*sync_entries).get(&id) {
        Some(sync) => { 
            return Some(sync.clone());
         },
        None => { return None; }
    }
}

#[tauri::command]
fn switch_sync(id: i64, database: tauri::State<Arc<Database>>) -> Option<bool> {
    println!("Switch sync: id: {}", id);
    
    let mut sync_entries = database.sync_entries.lock().unwrap();

    match (*sync_entries).get_mut(&id) {
        Some(sync) => { 
            sync.enabled = !(sync.enabled);
            return Some(sync.enabled);
         },
        None => { return None; }
    }
}

#[tauri::command]
fn validate_paths(path_from: &str, path_to: &str) -> Option<u32> {
    println!("Validating paths: {} - {}", path_from, path_to);

    let from_dir_valid = Path::new(path_from).is_dir();
    let to_dir_valid = Path::new(path_to).is_dir();
    
    let mut code = 0;

    if !from_dir_valid {
        code |= 1 << 1;
    }
    if !to_dir_valid {
        code |= 1 << 2;
    }
    if code == 0 && path_from == path_to {
        code |= 1 << 3;
    }

    //Add a warning if a sync with the same paths exists

    if code == 0 {
        return None;
    }

    Some(code)
}

#[tauri::command]
fn get_next_id(database: tauri::State<Arc<Database>>) -> i64 {
    let mut next_id = database.next_id.lock().unwrap();

    let current_id = *next_id;
    *next_id = *next_id + 1;

    println!("Returning next id: {}", current_id);
    return current_id;
}

#[tauri::command]
fn save_edited_id(id: i64, database: tauri::State<Arc<Database>>) {
    let mut save_edited_id = database.edited_id.lock().unwrap();
    (*save_edited_id) = Some(id);

    println!("Edited ID saved: {}", (*save_edited_id).unwrap_or(-1));
}

#[tauri::command]
fn reset_edit(database: tauri::State<Arc<Database>>) {
    let mut save_edited_id = database.edited_id.lock().unwrap();
    (*save_edited_id) = None;

    println!("Edited ID reset: {}", (*save_edited_id) == None);
}

#[tauri::command]
fn is_edited(database: tauri::State<Arc<Database>>) -> Option<i64> {
    let save_edited_id = database.edited_id.lock().unwrap();

    *save_edited_id
}

fn load_data_from_db() -> Arc<Database> {
    let local_data_base = Database{
                                                sync_entries: Mutex::new(HashMap::new()),
                                                next_id: Mutex::new(0),
                                                edited_id: Mutex::new(None) 
                                            };

    Arc::new(local_data_base)
}

fn sync_folders(database: Arc<Database>) {
    for i in 1..10 {
        println!("hi number {} from the spawned thread!", i);
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    let local_data_base = load_data_from_db();

    let local_db_update = Arc::clone(&local_data_base);
    let local_db_tauri = Arc::clone(&local_data_base);

    let sync_update_handle = thread::spawn(move ||{
        sync_folders(local_db_update);
    });

    tauri::Builder::default()
        .manage(local_db_tauri)
        .invoke_handler(tauri::generate_handler![
            add_sync, delete_sync, replace_sync, get_sync, switch_sync,
            validate_paths, get_next_id, save_edited_id, reset_edit,
            is_edited])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    sync_update_handle.join().unwrap();
}
