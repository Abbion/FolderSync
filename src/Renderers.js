//Constants
const defaultPath = "Select folder";
const defaultIntervalValue = 10;
const syncEnableColor = "#05A66B"
const syncDisabledColor = "#262626"
const syncLockColor = "#D92B2B"

function openEditBox(){
    let editBox = document.getElementById('edit-box');
  
    if(editBox)
      editBox.style.visibility = "visible";
    else
      console.error("Html element not found: " + editBox);
  }

function closeEditBox(){
    let editBox = document.getElementById('edit-box');
  
    if(editBox){
      let filePaths = editBox.getElementsByTagName("p1");

      for (let path of filePaths){
        path.innerHTML = defaultPath;
        path.title = "";
      }
      console.log(filePaths);
  
      editBox.style.visibility = "hidden";
    }
    else
      console.error("Html element not found: " + editBox);
  
    let intervalValue = document.getElementById('interval-value');
  
    if (intervalValue){
      intervalValue.value = defaultIntervalValue;
    }
    else
      console.error("interval value not found");
  }

  async function renderNewRecord(syncData, id){  
    const entryHtml = createNewSyncEntryHtml(syncData.paths, id);
  
    let syncTable = document.getElementById("sync-table");
  
    if (syncTable) {
      syncTable.innerHTML += entryHtml;
    }
    else {
        console.error("Html element not found: " + syncTable);
    }
  }

  async function renderUpdatedRecord(syncData, id){  
    let from_path = document.getElementById("from-folder-" + id);
    let to_path = document.getElementById("to-folder-" + id);

    console.log(syncData);

    if (from_path && to_path) {
      from_path.innerHTML = syncData.paths[0];
      to_path.innerHTML = syncData.paths[1];
    }
    else {
      console.error("Html element not found: " + from_path);
      console.error("Html element not found: " + to_path);
    }
  }

  function updateSyncStateColor(state, id) {
    let stateIndicator = document.getElementById("state-" + id);
    
    if (stateIndicator) {
      switch (state) {
        case "ENABLED":
          stateIndicator.style.backgroundColor = syncEnableColor;
          stateIndicator.title = "";
          break;
        case "DISABLED":
          stateIndicator.style.backgroundColor = syncDisabledColor;
          stateIndicator.title = "";
          break
        case "LOCKED":
          stateIndicator.style.backgroundColor = syncLockColor;
          stateIndicator.title = "One of the paths is damaged. Edit the record to update it";
          break
      }
    }
    else {
      console.error("Html element not found: " + stateIndicator);
    }
  }