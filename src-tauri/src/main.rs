// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::path::Path;
use std::{thread, string};
use std::time::{Duration, SystemTimeError, SystemTime};

use serde::Serialize;
use sqlite::State;
use tauri::Manager;

#[derive(serde::Deserialize, Debug, Clone, Serialize)]
enum IntervalType {
    SECOND = 0,
    MINUTE = 1,
    HOUR = 2
}

trait ToIntervalType {
    fn to_interval_type(self) -> IntervalType;
}

impl ToIntervalType for i64 {
    fn to_interval_type(self) -> IntervalType {
        match self {
            0 => IntervalType::SECOND,
            1 => IntervalType::MINUTE,
            2 => IntervalType::HOUR,
            _ => IntervalType::SECOND,
        }
    }
}
#[derive(serde::Deserialize, Debug, Clone, Serialize)]
struct SyncData {
    from_path: String,
    to_path: String,
    interval_value: u64,
    interval_time: u64,
    interval_type: IntervalType,
    enabled: bool
}

struct Database {
    sync_entries: Mutex<HashMap<u64, SyncData>>,
    next_id: Mutex<u64>,
    edited_id: Mutex<Option<u64>>,
    sql_connection: Mutex<sqlite::Connection>
}

impl Database {
    fn new() -> Database {
        Database{
            sync_entries: Mutex::new(HashMap::new()),
            next_id: Mutex::new(0),
            edited_id: Mutex::new(None),
            sql_connection : Mutex::new(sqlite::open("sync.db").unwrap())
        }
    }
}

#[tauri::command]
fn add_sync(sync_data: SyncData, id: u64, database: tauri::State<Arc<Database>>) -> bool {
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
fn delete_sync(id: u64, database: tauri::State<Arc<Database>>) -> bool {
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
fn replace_sync(mut sync_data: SyncData, id: u64, database: tauri::State<Arc<Database>>) -> bool {
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
                    from_path = {},
                    to_path = {},
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
fn get_sync(id: u64, database: tauri::State<Arc<Database>>) -> Option<SyncData> {
    let sync_entries = database.sync_entries.lock().unwrap();

    match (*sync_entries).get(&id) {
        Some(sync) => { 
            return Some(sync.clone());
         },
        None => { return None; }
    }
}

#[tauri::command]
fn switch_sync(id: u64, database: tauri::State<Arc<Database>>) -> Option<bool> {
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
fn get_next_id(database: tauri::State<Arc<Database>>) -> u64 {
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
fn save_edited_id(id: u64, database: tauri::State<Arc<Database>>) {
    let mut save_edited_id = database.edited_id.lock().unwrap();
    (*save_edited_id) = Some(id);

    println!("Edited ID saved: {:?}", (*save_edited_id));
}

#[tauri::command]
fn reset_edit(database: tauri::State<Arc<Database>>) {
    let mut save_edited_id = database.edited_id.lock().unwrap();
    (*save_edited_id) = None;

    println!("Edited ID reset: {}", (*save_edited_id) == None);
}

#[tauri::command]
fn is_edited(database: tauri::State<Arc<Database>>) -> Option<u64> {
    let save_edited_id = database.edited_id.lock().unwrap();

    *save_edited_id
}

#[tauri::command]
fn get_loaded_sync(database: tauri::State<Arc<Database>>) -> HashMap<u64, SyncData> {
    let sync_entries = database.sync_entries.lock().unwrap();
    (*sync_entries).clone()
}

fn load_data_from_db() -> Arc<Database> {
    let local_data_base = Database::new();
    
    {
        let connection = local_data_base.sql_connection.lock().unwrap();

        //Table creation
        let create_tables_query = "
            CREATE TABLE IF NOT EXISTS sync (
                id INTEGER PRIMARY KEY,
                from_path TEXT,
                to_path TEXT,
                interval_value INTEGER,
                interval_type INTEGER,
                enabled BOOLEAN
            );
            CREATE TABLE IF NOT EXISTS id_generator (
                id INTEGER PRIMARY KEY,
                next_sync_id INTEGER
            );
        ";

        connection.execute(create_tables_query).unwrap();

        //Id Generator
        let get_next_id_query = "SELECT COUNT(*) AS row_count FROM id_generator;";

        connection
        .iterate(get_next_id_query, |row| {
            let row_count : u64 = row[0].1.unwrap().parse().unwrap();
            
            if row_count == 0 {
                let initialize_id_generator = "INSERT INTO id_generator VALUES (0, 0);";
                connection.execute(initialize_id_generator).unwrap();
            } else {
                let get_id_generator = "SELECT * FROM id_generator WHERE id = 0";

                connection.iterate(get_id_generator, |row| {
                    let id_generator_value : u64 = row[1].1.unwrap().parse().unwrap();
                    let mut next_id = local_data_base.next_id.lock().unwrap();
                    (*next_id) = id_generator_value;

                    println!("Next id form db {}", (*next_id));
                    true
                }).unwrap();
            }

            true
        })
        .unwrap();

        //Get sync
        let get_all_sync_query = "SELECT * FROM sync";
        let mut statement = connection.prepare(get_all_sync_query).unwrap();

        let mut sync_entries = local_data_base.sync_entries.lock().unwrap();

        while let Ok(State::Row) = statement.next() {
            let sync_entry = SyncData{
                                        from_path: statement.read::<String, _>("from_path").unwrap(),
                                        to_path: statement.read::<String, _>("to_path").unwrap(),
                                        interval_value: statement.read::<i64, _>("interval_value").unwrap() as u64,
                                        interval_time: 0,
                                        interval_type: statement.read::<i64, _>("interval_type").unwrap().to_interval_type(),
                                        enabled: if statement.read::<i64, _>("enabled").unwrap() > 0 { true } else { false }
                            };
            (*sync_entries).insert(statement.read::<i64, _>("id").unwrap() as u64, sync_entry);
        }

        println!("{:?}", sync_entries);
    }

    Arc::new(local_data_base)
}

fn sync_folders(database: Arc<Database>, should_close: Arc<Mutex<bool>>) {
    let timer = SystemTime::now();
    let mut last_recorded_seconds :u64 = 0;
    let mut delta_time : u64 = 0;

    while !(*should_close.lock().unwrap()) {
        match timer.elapsed() {
            Ok(elapsed) => {
                let elapsed_seconds = elapsed.as_secs();
                delta_time = elapsed_seconds - last_recorded_seconds;
                last_recorded_seconds = elapsed_seconds;
            }
            Err(e) => {
                println!("Timer error: {e:?}");
            }
        }

        for (id, entry) in database.sync_entries.lock().unwrap().iter_mut() {
            if entry.enabled == false {
                continue;
            }

            entry.interval_time += delta_time;
            let interval_in_seconds = convert_to_seconds(entry.interval_type.clone(), entry.interval_value);

            if entry.interval_time > interval_in_seconds {
                entry.interval_time = 0;

                let from_path = entry.from_path.clone();
                let to_path = entry.to_path.clone();

                println!("coppying {} to {}", from_path, to_path);
            }
        }
    }
}

fn convert_to_seconds(interval_type: IntervalType, value: u64) -> u64 {
    match interval_type {
        IntervalType::SECOND => { return value; }
        IntervalType::MINUTE => { return value * 60; }
        IntervalType::HOUR => { return value * 60 * 60; }
    }
}

#[derive(Clone, Serialize)]
struct Payload {
    message: String
}

fn main() {
    let local_data_base = load_data_from_db();

    let local_db_update = Arc::clone(&local_data_base);
    let local_db_tauri = Arc::clone(&local_data_base);

    let tauri_closed = Arc::new(Mutex::new(false));
    let tauri_closed_update = Arc::clone(&tauri_closed);

    let sync_update_handle =  Arc::new(Mutex::new(Some(thread::spawn(move ||{
        sync_folders(local_db_update, tauri_closed_update);
    }))));

    tauri::Builder::default()
        .manage(local_db_tauri)
        .invoke_handler(tauri::generate_handler![
            add_sync, delete_sync, replace_sync, get_sync, switch_sync,
            validate_paths, get_next_id, save_edited_id, reset_edit,
            is_edited, get_loaded_sync])
        .on_window_event(move |event| match event.event()  {
            tauri::WindowEvent::Destroyed => {
                (*tauri_closed.lock().unwrap()) = true;

                if let Some(handle) = sync_update_handle.lock().unwrap().take() {
                    handle.join().unwrap();
                    println!("Update shutdown");
                }
                println!("Tauri shutdown");
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
