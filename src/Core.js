class Mutex {
    constructor() {
        this.queue = [];
        this.locked = false;
    }

    async acquire() {
        const release = () => {
            const next = this.queue.shift();
            if (next) {
                next();
            } else {
                this.locked = false;
            }
        };

        if (this.locked) {
            await new Promise((resolve) => {
                this.queue.push(resolve);
            });
        } else {
            this.locked = true;
        }

        return release;
    }
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}