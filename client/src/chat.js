// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { sendNotification } = window.__TAURI__.notification;
const { appWindow } = window.__TAURI__.window;

function sendMessage(body, channel) {
    invoke('send_message', { message: body, channel: channel })
}

// Define an async function to use await
async function init() {
    // invoke('greet', { name: 'World' })
    //     // `invoke` returns a Promise
    //     .then((response) => {
    //         window.header.innerHTML = response;
    //     });

    // Now you can use await within an async function
    await listen('recieve_message', (event) => {
        console.log("message received: " + JSON.stringify(event.payload["Text"]));
        let input = event.payload["Text"];

        const chatElem = document.getElementById("inner-messages");
        const scrollElem = document.getElementById("messages");

        var para = document.createElement("p");
        para.innerHTML = '<strong class="who">' + input.username + ': </strong>' + input.body;
        chatElem.appendChild(para); // to be moved to listen event

        scrollElem.scrollTop = scrollElem.scrollHeight;
    });

    await listen('init_channels', (event) => {
        console.log("channels received: ");

        const channelOptionList = document.getElementById("channels");

        for (const element of event.payload) {
            console.log(element);

            var channelOption = document.createElement("option");
            channelOption.innerHTML = element;
            channelOptionList.appendChild(channelOption);
        }
    });

    invoke('request_channels');
    invoke('request_history', { channel: 'general', amount: '50' });
}

// Call the async function to start the initialization process
init().catch(console.error);
