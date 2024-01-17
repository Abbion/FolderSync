const mutex = new Mutex();

async function showError(message) {
    let infoBox = document.getElementById('info-box');
    let infoBoxTile = document.getElementById('info-box-title');
    let infoBoxMessage = document.getElementById('info-box-message');

    const release = await mutex.acquire();

    if (infoBox && infoBoxTile && infoBoxMessage) {
        infoBoxTile.innerHTML = "Error!";
        infoBoxTile.classList.add('error-box-title');

        infoBoxMessage.innerHTML = message;

        infoBox.classList.add('popupInfoBox');

        sleep(4000).then(() => {
            infoBox.classList.remove('popupInfoBox');
            infoBoxTile.classList.remove('error-box-title');
            sleep(500).then(() => {
                release();
            });
        });
    }
}

async function showWarning(message) {
    let infoBox = document.getElementById('info-box');
    let infoBoxTile = document.getElementById('info-box-title');
    let infoBoxMessage = document.getElementById('info-box-message');

    const release = await mutex.acquire();

    if (infoBox && infoBoxTile && infoBoxMessage) {
        infoBoxTile.innerHTML = "Warning!";
        infoBoxTile.classList.add('warning-box-title');

        infoBoxMessage.innerHTML = message;
        
        infoBox.classList.add('popupInfoBox');

        sleep(4000).then(() => {
            infoBox.classList.remove('popupInfoBox');
            infoBoxTile.classList.remove('warning-box-title');
            sleep(500).then(() => {
                release();
            });
        });
    }
}

function pathValidationErrorIdToText(errorCode) {
    const errorMessages = [];

    if (errorCode & (1 << 1)) errorMessages.push("From path is not valid!");
    if (errorCode & (1 << 2)) errorMessages.push("To path is not valid!");
    if (errorCode & (1 << 3)) errorMessages.push("Paths are the same. Cannot create copy loop!");
    if (errorCode & (1 << 4)) errorMessages.push("Entry with those paths already exists");

    return errorMessages;
}