const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { message } = window.__TAURI__.dialog;

function connectServer(ip, user, pass) {
    return new Promise((resolve, reject) => {
        invoke('connect_server', { ip: ip, username: user, password: pass })
            .then(() => {
                resolve();
            })
            .catch((error) => {
                reject(error);
            });
    });
}

function handleConnection(ip, user, pass) {
    window.localStorage.setItem("username", user);
    connectServer(ip, user, pass).then(()=>{window.location.href="/chat.html"}).catch(e=>{console.error(e); message(e, { title: 'Tauri', type: 'error' })});
}