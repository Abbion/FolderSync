use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;

use crate::structs;
use structs::{*};

#[tauri::command]
pub fn add_sync(sync_data: SyncData, id: u64, database: tauri::State<Arc<Database>>) -> bool {
    println!("Add new sync: {:?}, id: {}", sync_data, id);

    let mut sync_entries = database.sync_entries.lock().unwrap();
    
    if (*sync_entries).contains_key(&id) {
        return false;
    }

    (*sync_entries).insert(id, sync_data.clone());

    let connection = database.sql_connection.lock().unwrap();
    let insert_sync_quary = format!("
                INSERT INTO sync (id, from_path, to_path, interval_value, interval_type, sync_state) 
                VALUES ({}, '{}', '{}', {}, {}, {});
                ", id, sync_data.from_path, sync_data.to_path, sync_data.interval_value, sync_data.interval_type as u8, sync_data.sync_state as u8);

    println!("{}", insert_sync_quary);

    connection.execute(insert_sync_quary).unwrap();

    return true;
}

#[tauri::command]
pub fn replace_sync(mut sync_data: SyncData, id: u64, database: tauri::State<Arc<Database>>) -> bool {
    println!("Replace sync: {:?}, id: {}", sync_data, id);
    
    let mut sync_entries = database.sync_entries.lock().unwrap();
    
    if (*sync_entries).contains_key(&id) {
        let old_enabled = (*sync_entries).get(&id).unwrap().sync_state.clone();
        (*sync_entries).remove(&id);

        if old_enabled != SyncState::LOCKED {
            sync_data.sync_state = old_enabled;
        }
        else {
            sync_data.sync_state = SyncState::ENABLED;
        }

        (*sync_entries).insert(id, sync_data.clone());
        
        let connection = database.sql_connection.lock().unwrap();
        let update_sync_quary = format!("
                    UPDATE sync SET
                    from_path = '{}',
                    to_path = '{}',
                    interval_value = {},
                    interval_type = {},
                    sync_state = {}
                    WHERE id = {};",
                    sync_data.from_path, sync_data.to_path, sync_data.interval_value, sync_data.interval_type as u8, sync_data.sync_state as u8,  id
                );
        
        connection.execute(update_sync_quary).unwrap();

        return true;
    }

    return false;
}

#[tauri::command]
pub fn delete_sync(id: u64, database: tauri::State<Arc<Database>>) -> bool {
    println!("Delete sync:, id: {}", id);

    let mut sync_entries = database.sync_entries.lock().unwrap();
    
    if (*sync_entries).contains_key(&id) {
        (*sync_entries).remove(&id);

        let connection = database.sql_connection.lock().unwrap();
        let remove_sync_quary = format!("DELETE FROM sync WHERE id = {};", id);
        connection.execute(remove_sync_quary).unwrap();

        return true;
    }

    return false;
}

#[tauri::command]
pub fn get_sync(id: u64, database: tauri::State<Arc<Database>>) -> Option<SyncData> {
    let sync_entries = database.sync_entries.lock().unwrap();

    match (*sync_entries).get(&id) {
        Some(sync) => { 
            return Some(sync.clone());
         },
        None => { return None; }
    }
}

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

#[tauri::command]
pub fn switch_sync(id: u64, database: tauri::State<Arc<Database>>) -> Option<bool> {
    println!("Switch sync: id: {}", id);
    
    let mut sync_entries = database.sync_entries.lock().unwrap();

    match (*sync_entries).get_mut(&id) {
        Some(sync) => { 
            if sync.sync_state == SyncState::DISABLED {
                sync.sync_state = SyncState::ENABLED;
            }
            else if sync.sync_state == SyncState::ENABLED {
                sync.sync_state = SyncState::DISABLED;
            }

            let connection = database.sql_connection.lock().unwrap();
            let update_sync_quary = format!("
                        UPDATE sync SET
                        sync_state = {}
                        WHERE id = {};",
                        sync.sync_state.clone() as u8,  id
                    );
    
            connection.execute(update_sync_quary).unwrap();

            return Some(if sync.sync_state == SyncState::ENABLED { true } else { false });
         },
        None => { return None; }
    }
}

#[tauri::command] 
pub fn lock_sync(id: u64, database: tauri::State<Arc<Database>>) {
    println!("Lock sync: id: {}", id);

    let mut sync_entries = database.sync_entries.lock().unwrap();

    match (*sync_entries).get_mut(&id) {
        Some(sync) => { 
            let connection = database.sql_connection.lock().unwrap();
            let update_sync_quary = format!("
                        UPDATE sync SET
                        sync_state = {}
                        WHERE id = {};",
                        sync.sync_state.clone() as u8,  id
                    );
    
            connection.execute(update_sync_quary).unwrap();
         },
        None => { }
    }
}

#[tauri::command]
pub fn is_locked(id: u64, database: tauri::State<Arc<Database>>) -> bool {
    let mut sync_entries = database.sync_entries.lock().unwrap();

    match (*sync_entries).get_mut(&id) {
        Some(sync) => {
            if sync.sync_state == SyncState::LOCKED { true } else { false }
         },
        None => { false }
    }
}

#[tauri::command]
pub fn validate_paths(database: tauri::State<Arc<Database>>, path_from: &str, path_to: &str) -> Option<u32> {
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
    
    let edit_state = database.edited_id.lock().unwrap();

    if edit_state.is_none() {
        let sync_entries = database.sync_entries.lock().unwrap();
        for (_, entry) in (*sync_entries).iter() {
            if entry.from_path == path_from && entry.to_path == path_to {
                code |= 1 << 4;
                break;
            }
        }
    }

    if code == 0 {
        return None;
    }

    Some(code)
}

#[tauri::command]
pub fn get_next_id(database: tauri::State<Arc<Database>>) -> u64 {
    let mut next_id = database.next_id.lock().unwrap();

    let current_id = *next_id;
    *next_id = *next_id + 1;

    let connection = database.sql_connection.lock().unwrap();
    let update_sync_quary = format!("
                UPDATE id_generator SET
                next_sync_id = {}
                WHERE id = 0;",
                (*next_id)
            );

    connection.execute(update_sync_quary).unwrap();

    println!("Returning next id: {}", current_id);
    return current_id;
}

#[tauri::command]
pub fn save_edited_id(id: u64, database: tauri::State<Arc<Database>>) {
    let mut save_edited_id = database.edited_id.lock().unwrap();
    (*save_edited_id) = Some(id);

    println!("Edited ID saved: {:?}", (*save_edited_id));
}

#[tauri::command]
pub fn reset_edit(database: tauri::State<Arc<Database>>) {
    let mut save_edited_id = database.edited_id.lock().unwrap();
    (*save_edited_id) = None;

    println!("Edited ID reset: {}", (*save_edited_id) == None);
}

#[tauri::command]
pub fn is_edited(database: tauri::State<Arc<Database>>) -> Option<u64> {
    let save_edited_id = database.edited_id.lock().unwrap();

    *save_edited_id
}

#[tauri::command]
pub fn get_loaded_sync(database: tauri::State<Arc<Database>>) -> HashMap<u64, SyncData> {
    let sync_entries = database.sync_entries.lock().unwrap();
    (*sync_entries).clone()
}