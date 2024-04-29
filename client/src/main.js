const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { message } = window.__TAURI__.dialog;

function connectServer(ip) {
    return new Promise((resolve, reject) => {
        invoke('connect_server', { ip: ip })
            .then(() => {
                resolve();
            })
            .catch((error) => {
                reject(error);
            });
    });
}

function handleConnection(ip) {
    connectServer(ip).then(()=>{window.location.href="/chat.html"}).catch(e=>{console.error(e); message(e, { title: 'Tauri', type: 'error' })});
}