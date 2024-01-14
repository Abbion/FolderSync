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
                INSERT INTO sync (id, from_path, to_path, interval_value, interval_type, enabled) 
                VALUES ({}, '{}', '{}', {}, {}, {});
                ", id, sync_data.from_path, sync_data.to_path, sync_data.interval_value, sync_data.interval_type as u8, sync_data.enabled);

    connection.execute(insert_sync_quary).unwrap();

    return true;
}

#[tauri::command]
pub fn replace_sync(mut sync_data: SyncData, id: u64, database: tauri::State<Arc<Database>>) -> bool {
    println!("Replace sync: {:?}, id: {}", sync_data, id);
    
    let mut sync_entries = database.sync_entries.lock().unwrap();
    
    if (*sync_entries).contains_key(&id) {
        let old_enabled = (*sync_entries).get(&id).unwrap().enabled;
        (*sync_entries).remove(&id);

        sync_data.enabled = old_enabled;
        (*sync_entries).insert(id, sync_data.clone());
        
        let connection = database.sql_connection.lock().unwrap();
        let update_sync_quary = format!("
                    UPDATE sync SET
                    from_path = '{}',
                    to_path = '{}',
                    interval_value = {},
                    interval_type = {},
                    enabled = {}
                    WHERE id = {};",
                    sync_data.from_path, sync_data.to_path, sync_data.interval_value, sync_data.interval_type as u8, sync_data.enabled,  id
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

#[tauri::command]
pub fn switch_sync(id: u64, database: tauri::State<Arc<Database>>) -> Option<bool> {
    println!("Switch sync: id: {}", id);
    
    let mut sync_entries = database.sync_entries.lock().unwrap();

    match (*sync_entries).get_mut(&id) {
        Some(sync) => { 
            sync.enabled = !(sync.enabled);

            let connection = database.sql_connection.lock().unwrap();
            let update_sync_quary = format!("
                        UPDATE sync SET
                        enabled = {}
                        WHERE id = {};",
                        sync.enabled,  id
                    );
    
            connection.execute(update_sync_quary).unwrap();

            return Some(sync.enabled);
         },
        None => { return None; }
    }
}

#[tauri::command]
pub fn validate_paths(path_from: &str, path_to: &str) -> Option<u32> {
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