const syncEntryTemplate = `
<div id="sync-entry-{{ id }}" class="sync-entry">
    <div class="state" onclick="enableSync({{ id }})">
    </div>

    <p2 class="from-folder" title="{{ fromFolderPath }}">
        {{fromFolderPath}}
    </p2>

    <p2 class="to-folder" title="{{ toFolderPath }}">
        {{toFolderPath}}
    </p2>
    
    <div class="sync-icons edit-icon" onclick="editRecord({{ id }})">
        <img src="assets/Edit.svg"/>
    </div>

    <div class="sync-icons delete-icon" onclick="deleteRecord({{ id }})">
        <img src="assets/Delete.svg"/>
    </div>
</div>
`

function createNewSyncEntryHtml(paths, id){
    const syncEntryTemplateData = {
        id: id,
        fromFolderPath: paths[0],
        toFolderPath: paths[1]
    };

    return syncEntryTemplate.replace(/{{\s*([\w.]+)\s*}}/g, (matched, key) => {
        if (syncEntryTemplateData.hasOwnProperty(key)) {
            return syncEntryTemplateData[key];
        } else {
            //Return the original value
            return matched;
        }
    });
}