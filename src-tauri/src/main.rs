// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Mutex, Arc, mpsc};
use std::{thread, fs};
use std::time::SystemTime;

use sqlite::State;

mod vec_helper;
mod sys_helper;
mod structs;
mod tauri_commands;

use tauri::Manager;
use tauri_commands::{*};
use structs::{*};

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
                sync_state INTEGER
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
                                        id: statement.read::<i64, _>("id").unwrap() as u64,
                                        from_path: statement.read::<String, _>("from_path").unwrap(),
                                        to_path: statement.read::<String, _>("to_path").unwrap(),
                                        interval_value: statement.read::<i64, _>("interval_value").unwrap() as u64,
                                        interval_time: 0,
                                        interval_type: statement.read::<i64, _>("interval_type").unwrap().to_interval_type(),
                                        sync_state: statement.read::<i64, _>("sync_state").unwrap().to_sync_state_type()
                            };
            (*sync_entries).insert(statement.read::<i64, _>("id").unwrap() as u64, sync_entry);
        }

        println!("{:?}", sync_entries);
    }

    Arc::new(local_data_base)
}

fn sync_folders(database: Arc<Database>, should_close: Arc<Mutex<bool>>, sender: mpsc::Sender<AppEvent>) {
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

        for (_, entry) in database.sync_entries.lock().unwrap().iter_mut() {
            if entry.sync_state != SyncState::ENABLED {
                continue;
            }

            entry.interval_time += delta_time;
            let interval_in_seconds = convert_to_seconds(entry.interval_type.clone(), entry.interval_value);

            if entry.interval_time > interval_in_seconds {
                entry.interval_time = 0;

                let from_path = entry.from_path.clone();
                let to_path = entry.to_path.clone();
                
                let from_folder_exists = sys_helper::check_if_folder_exit(&from_path);
                let to_folder_exists = sys_helper::check_if_folder_exit(&to_path);

                
                if from_folder_exists == false || to_folder_exists == false {
                    entry.sync_state = SyncState::LOCKED;



                    sender.send(AppEvent{ event_code: EventCode::FolderNotExisting(entry.id) }).unwrap();
                    continue;
                }

                let from_files = sys_helper::get_file_names_from_folder(&from_path);
                let to_files = sys_helper::get_file_names_from_folder(&to_path);   

                //Update shared files
                let shared_files_list = vec_helper::find_intersection(&from_files, &to_files);
    
                for shared_file in shared_files_list {
                    let source_path = format!("{}\\{}", from_path, shared_file);
                    let destination_path = format!("{}\\{}", to_path, shared_file);

                    let source_mod_date = sys_helper::get_file_modification_date(&source_path);
                    let destination_mod_date = sys_helper::get_file_modification_date(&destination_path);

                    if let Some(s_date) = source_mod_date {
                        if let Some(d_date) = destination_mod_date {
                            let s_duration = s_date.elapsed().unwrap();
                            let d_duration = d_date.elapsed().unwrap();

                            if s_duration.as_millis() != d_duration.as_millis() {
                                println!("Copying {} -> {}", source_path, destination_path);

                                match fs::copy(source_path, destination_path) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        println!("Error while copying files: {}", e);
                                    }
                                }
                            }

                        } else {
                          continue;  
                        }
                    } else {
                        continue;
                    }
                }

                //Copy missing files
                let missing_files_list = vec_helper::find_difference(&from_files, &to_files);

                for missing_file in missing_files_list {
                    let source_path = format!("{}\\{}", from_path, missing_file);
                    let destination_path = format!("{}\\{}", to_path, missing_file);

                    println!("Copying {} -> {}", source_path, destination_path);

                    match fs::copy(source_path, destination_path) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Error while copying files: {}", e);
                        }
                    }
                }
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

enum EventCode {
    FolderNotExisting(u64),
}

struct AppEvent {
    event_code: EventCode,
}

#[derive(Clone, serde::Serialize)]
struct Payload {
  id: u64,
}

fn main() {
    let local_data_base = load_data_from_db();

    let local_db_update = Arc::clone(&local_data_base);
    let local_db_tauri = Arc::clone(&local_data_base);

    let tauri_closed = Arc::new(Mutex::new(false));
    let tauri_closed_update = Arc::clone(&tauri_closed);

    let (tx, rx) = mpsc::channel::<AppEvent>();

    let sync_update_handle =  Arc::new(Mutex::new(Some(thread::spawn(move ||{
        sync_folders(local_db_update, tauri_closed_update, tx);
    }))));

    tauri::Builder::default()
        .setup(|app|{

            let handle_for_thread = app.handle();

            thread::spawn(move || {
                for received in rx {
                    match received.event_code {
                        EventCode::FolderNotExisting(id) => {
                            handle_for_thread.emit_all("folder-not-existing-event", Payload{id: id}).unwrap();
                        }
                    }
                }
            });

            Ok(())
        })
        .manage(local_db_tauri)
        .invoke_handler(tauri::generate_handler![
            add_sync, delete_sync, replace_sync, get_sync, switch_sync, lock_sync,
            is_locked, validate_paths, get_next_id, save_edited_id, reset_edit,
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