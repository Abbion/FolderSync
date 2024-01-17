const { open } = window.__TAURI__.dialog;

function setData(data) {
  let editBox = document.getElementById('edit-box');
  let paths = [];

  if (editBox) {
    let filePaths = editBox.getElementsByTagName("p1");

    paths.push(data.from_path);
    paths.push(data.to_path);

    let itr = 0;
    for (let path of filePaths){
      path.innerHTML = paths[itr];
      path.title = paths[itr];
      itr++;
    }
  }
  else {
    console.error("Html element not found: " + editBox);
    return {};
  }

  let intervalInput = document.getElementById('interval-value');

  if (intervalInput) {
    intervalInput.value = data.interval_value;
  }
  else {
    console.error("Html element not found: " + intervalInput);
    return {};
  }

  let intervalSelector = document.getElementById('interval-type');

  if (intervalSelector) {
    intervalSelector.value = data.interval_type.toLowerCase();
  }
  else {
    console.error("Html element not found: " + intervalInput);
    return {};
  }
}

function getData() {
  let editBox = document.getElementById('edit-box');
  let paths = [];
  let intervalValue;
  let intervalType;

  if (editBox) {
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

    if (intervalInput) {
      intervalValue = parseInt(intervalInput.value);
    }
    else {
      console.error("Element: " + intervalInput);
      return {};
    }

    let intervalSelector = document.getElementById('interval-type');

    if (intervalSelector) {
      intervalType = intervalSelector.value;
    }
    else {
      console.error("Element: " + intervalInput);
      return {};
    }

    return {
      paths: paths,
      intervalValue: intervalValue,
      intervalType: intervalType
    };
  }
  else {
    console.error("Html element not found: " + editBox);
    return {};
  }
}

async function addSyncNewPressed() {
  await resetEdit();
  openEditBox();
}

async function saveRecord() {
  const data = getData();

  if (Object.keys(data).length != 3) {
    showError("Incorrect amount of keys")
    return;
  }

  const isEditedPromise = isEdited();
  const valid = await validatePaths(data.paths[0], data.paths[1])

  if (valid == null) {
    let id = await isEditedPromise;

    if (id == null) {
      id = await getNextID();
      const add_state = await addSync(data, id);

      if (add_state) {
        renderNewRecord(data, id);
      }
      else {
        showError("Adding new record failed!");
      }
    }
    else {
      const lockStatePromise = isSyncLocked(id);
      const replace_state = await replaceSync(data, id);

      console.log("replace_state ", replace_state);

      if (replace_state) {
        renderUpdatedRecord(data, id);

        if (await lockStatePromise) {
          updateSyncStateColor("ENABLED", id);
        }
      }
      else {
        showError("Replacing record failed!");
      }
    }

    closeEditBox();
  }
  else{
    const errorMessages = pathValidationErrorIdToText(valid);

    for (const errorMessage of errorMessages) {
      showError(errorMessage);
    }
  }
}

async function editRecord(id) {
  await saveEditedID(id);

  let syncData = getSync(id);
  openEditBox();
  setData(await syncData);
}

async function deleteRecord(id) {
  const result = await deleteSync(id);

  if (result) {
    const syncEntry = document.getElementById("sync-entry-" + id);

    if (syncEntry){
      syncEntry.remove();
    } 
    else {
      console.error("Html element not found: " + syncEntry);
    }
  }
  else {
    showError("Deleting record failed!");
  }
}

async function enableSync(id) {
  if (await isSyncLocked(id)) {
    showError("One of the paths is damaged. Edit the record to update it");
    return
  }

  const newStatePromise = await switchSync(id);
  updateSyncStateColor(newStatePromise ? "ENABLED" : "DISABLED", id);
}

async function openFolder(htmlElement) {
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

window.addEventListener('DOMContentLoaded', async () => {
  let syncMap = await getLoadedSync();

  for (const id in syncMap) {
    const item = syncMap[id];

    let syncToRender = {
      paths: [item.from_path, item.to_path],
      intervalValue: item.interval_value,
      intervalType: item.interval_type
    }

    renderNewRecord(syncToRender, id);
    updateSyncStateColor(item.sync_state , id);
  };
});
