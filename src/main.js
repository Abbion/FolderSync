//let greetInputEl;
//let greetMsgEl;

const { invoke } = window.__TAURI__.tauri;
const { open } = window.__TAURI__.dialog;

//Rust handlers
async function addNewSync(syncData, id) {
  return await invoke("add_sync", { syncData: { from_path: syncData.paths[0],
                                                to_path: syncData.paths[1],
                                                interval_value: syncData.intervalValue,
                                                interval_type: syncData.intervalType.toUpperCase()
                                              },
                                    id: id });
}

async function deleteSync(id) {
  return await invoke("delete_sync", { id: id });
}

async function replaceSync(syncData, id) {
  return await invoke("replace_sync", { syncData: { from_path: syncData.paths[0],
                                                    to_path: syncData.paths[1],
                                                    interval_value: syncData.intervalValue,
                                                    interval_type: syncData.intervalType.toUpperCase()
                                              },
                                        id: id });
}

async function saveEditedID(id) {
  await invoke("save_edited_id", { editedId: id });
}

async function resetEdit() {
  await invoke("reset_edit");
}

async function getNextID() {
  return await invoke("get_next_id");
}

async function isEdited() {
  return await invoke("is_edited");
}

async function validatePaths(pathFrom, pathTo) {
  return await invoke("validate_paths", { pathFrom: pathFrom, pathTo: pathTo });
}

const defaultPath = "Select folder";
const defaultIntervalValue = 10;
const syncEnableColor = "#05A66B"
const syncDisabledColor = "#262626"

function closeEditBox(){
  let editBox = document.getElementById('edit-box');

  if(editBox){
    let filePaths = editBox.getElementsByTagName("p1");
    for (let path of filePaths){
      path.innerHTML = defaultPath
    }
    console.log(filePaths);

    editBox.style.visibility = "hidden";
  }
  else
    console.error("Path element not found: " + editBox);

  let intervalValue = document.getElementById('interval-value');

  if (intervalValue){
    intervalValue.value = defaultIntervalValue;
  }
  else
    console.error("interval value not found");
}

async function addSyncNewPressed() {
  await resetEdit();
  openEditBox();
}

function openEditBox(){
  let editBox = document.getElementById('edit-box');

  if(editBox)
    editBox.style.visibility = "visible";
  else
    console.error("Element: " + editBox);
}

function getData(){
  let editBox = document.getElementById('edit-box');
  let paths = [];
  let intervalValue;
  let intervalType;

  if(editBox){
    let filePaths = editBox.getElementsByTagName("p1");
    for (let path of filePaths){
      if(path.innerHTML.length === 0 || path.innerHTML.trim() === defaultPath){
        window.alert("empty path");
        return {};
      }
      else{
        paths.push(path.innerHTML.trim());
      }
    }
    
    let intervalInput = document.getElementById('interval-value');

    if(intervalInput){
      intervalValue = parseInt(intervalInput.value);
    }
    else{
      console.error("Element: " + intervalInput);
      return {};
    }

    let intervalSelector = document.getElementById('interval-type');

    if(intervalSelector){
      intervalType = intervalSelector.value;
    }
    else{
      console.error("Element: " + intervalInput);
      return {};
    }

    return {
      paths: paths,
      intervalValue: intervalValue,
      intervalType: intervalType
    };
  }
  else
    console.error("Element: " + editBox);

  return {};
}

async function addRecord(syncData, id){
  console.log("Add record: ", syncData);
  
  const add_state = await addNewSync(syncData, id);
  
  console.log("add state: " + add_state);

  //Create hmtl code
  if (add_state) {
    const entryHtml = createNewSyncEntryHtml(syncData.paths, id);
    let syncTable = document.getElementById("sync-table");

    if (syncTable) {
      syncTable.innerHTML += entryHtml;
    }
  }
  else {
    //Display adding error
  }
}

async function saveRecord() {
  console.log("Save record");
  const data = getData();

  console.log(data);

  if (Object.keys(data).length != 3){
    console.error("Incorrect amount of keys");
    return;
  }

  const isEditedPromise = isEdited();
  const valid = await validatePaths(data.paths[0], data.paths[1])

  if (valid == null) {
    let id = await isEditedPromise;

    if (id == null) {
      id = await getNextID();
      addRecord(data, id);
    }
    else {
      //replaceRecord();
    }
  }
  else{
    console.error("Paths are not valid code:", valid);
  }

  closeEditBox();
}

function deleteRecord(syncID) {
  console.log("Delete record")

  //Detete the sync from the database and state
  //Get the result 
  const result = true;
  const syncEntry = document.getElementById("sync-entry-" + syncID);

  if (syncEntry){
    syncEntry.remove();
  }

  //Find the entry and delete it
}

async function editRecord(syncID){
  console.log("Edit record");

  await saveEditedID(syncID);

  //Get the syncData from db
  const result = {
    paths: ["Test1", "Test2"],
    intervalValue: 420,
    intervalType: 'm'
  };

  openEditBox();

  //Fill the edit box with data
}

function enableSync(syncID){
  console.log("Enable sync");

  //Get the state for this sync
  //Return new state
  const newState = false;

  const syncEntry = document.getElementById("sync-entry-" + syncID);

  if (syncEntry){
    let state = syncEntry.getElementsByClassName("state")[0];
    
    if (state) {
      state.style.backgroundColor = (newState ? syncEnableColor : syncDisabledColor);
    }
  }
}

async function openFolder(htmlElement){
  console.log("Open folder");

  try{
    const selectedPath = await open({
      multiple: false,
      directory: true,
    });

    if (selectedPath === null)
      return;

    const htmlPathSelector = htmlElement.parentNode;

    if (!htmlPathSelector)
      throw new Error("Selector not found");

    const pathContainer = htmlPathSelector.querySelector('p1');

    if (!pathContainer)
      throw new Error("Path container not found");

    pathContainer.title = selectedPath;
    pathContainer.innerHTML = selectedPath;
  }
  catch (err){
    console.error("Open folder error: " + err);
  }
}

function validateIntervalValue(input) {
  // Remove non-numeric characters and leading zeros
  let sanitizedValue = input.value.replace(/[^0-9]/g, '').replace(/^0+/, '');
  
  sanitizedValue = sanitizedValue.slice(0, 3);
  input.value = sanitizedValue;
}

function checkIfValueIsEmpty(input){
  if (input.value.length === 0){
    input.value = "1";
  }
}

/*
async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
*/