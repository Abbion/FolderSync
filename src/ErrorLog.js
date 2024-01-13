const mutex = new Mutex();

async function showError(message) {
    let errorBox = document.getElementById('error-box');
    let errorBoxMessage = document.getElementById('error-box-message');

    const release = await mutex.acquire();

    if (errorBox && errorBoxMessage) {
        errorBoxMessage.innerHTML = message;
        
        errorBox.classList.add('popupErrorBox');

        sleep(4000).then(() => {
            errorBox.classList.remove('popupErrorBox');
            sleep(500).then(() => {
                release();
            });
        });
    }
}