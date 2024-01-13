//Constants
const defaultPath = "Select folder";
const defaultIntervalValue = 10;
const syncEnableColor = "#05A66B"
const syncDisabledColor = "#262626"

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

  async function renderRecord(syncData, id){  
    const entryHtml = createNewSyncEntryHtml(syncData.paths, id);
  
    let syncTable = document.getElementById("sync-table");
  
    if (syncTable) {
      syncTable.innerHTML += entryHtml;
    }
    else {
        console.error("Html element not found: " + syncTable);
    }
  }

  function updateSyncStateColor(state, id) {
    const syncEntry = document.getElementById("sync-entry-" + id);
  
    if (syncEntry){
      let stateIndicator = syncEntry.getElementsByClassName("state")[0];
      
      if (stateIndicator) {
        stateIndicator.style.backgroundColor = (state ? syncEnableColor : syncDisabledColor);
      }
      else {
        console.error("Html element not found: " + stateIndicator);
      }
    }
    else {
        console.error("Html element not found: " + syncEntry);
    }
  }