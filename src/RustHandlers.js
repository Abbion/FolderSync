const { invoke } = window.__TAURI__.tauri;
const { emit, listen } = window.__TAURI__.event

//Rust handlers
async function addSync(syncData, id) {
  return await invoke("add_sync", { syncData: {
                                                id: id,
                                                from_path: syncData.paths[0],
                                                to_path: syncData.paths[1],
                                                interval_value: syncData.intervalValue,
                                                interval_time: 0,
                                                interval_type: syncData.intervalType.toUpperCase(),
                                                sync_state: "ENABLED"
                                              },
                                    id: id });
}

async function deleteSync(id) {
  return await invoke("delete_sync", { id: id });
}

async function replaceSync(syncData, id) {
  return await invoke("replace_sync", { syncData: {
                                                    id: id,
                                                    from_path: syncData.paths[0],
                                                    to_path: syncData.paths[1],
                                                    interval_value: syncData.intervalValue,
                                                    interval_time: 0,
                                                    interval_type: syncData.intervalType.toUpperCase(),
                                                    sync_state: "ENABLED"
                                              },
                                        id: id });
}

async function getSync(id) {
  return await invoke("get_sync", { id: id });
}

async function switchSync(id) {
  return await invoke("switch_sync", { id: id });
}

async function lockSync(id) {
  return await invoke("lock_sync", {id: id});
}

async function isSyncLocked(id) {
  return await invoke("is_locked", {id: id});
}

async function validatePaths(pathFrom, pathTo) {
  return await invoke("validate_paths", { pathFrom: pathFrom, pathTo: pathTo });
}

async function getNextID() {
  return await invoke("get_next_id");
}

async function saveEditedID(id) {
  await invoke("save_edited_id", { id: id });
}

async function resetEdit() {
  await invoke("reset_edit");
}

async function isEdited() {
  return await invoke("is_edited");
}

async function getLoadedSync() {
  return await invoke("get_loaded_sync");
}

//Events
listen('folder-not-existing-event', (event) => {
  let id = event.payload.id;
  updateSyncStateColor("LOCKED", id);
  lockSync(id);
});