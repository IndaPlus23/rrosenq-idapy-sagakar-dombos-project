// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { sendNotification } = window.__TAURI__.notification;
const { appWindow } = window.__TAURI__.window;

function sendMessage(body) {
    const channelList = document.getElementById("channels");

    invoke('send_message', { message: body, channel: channelList.options[channelList.selectedIndex].value })
}

function channelChange() {
    const channelHolder = document.getElementById("all-channel-boxes");
    const channelList = document.getElementById("channels");

    const channels = channelHolder.getElementsByClassName("message-box");
    for (const channelBox of channels) {
        channelBox.style.display = "none";
    };

    const activeChannel = document.getElementById("channel-" + channelList.options[channelList.selectedIndex].value);
    activeChannel.style.display = "block";
    activeChannel.scrollTop = activeChannel.scrollHeight;
}

// Define an async function to use await
async function init() {
    // invoke('greet', { name: 'World' })
    //     // `invoke` returns a Promise
    //     .then((response) => {
    //         window.header.innerHTML = response;
    //     });

    await listen('recieve_message', (event) => {
        let input = event.payload["Text"];

        const scrollElem = document.getElementById("channel-" + input.channel);
        const chatElem = scrollElem.getElementsByClassName("inner-channel")[0];

        var para = document.createElement("p");
        para.innerHTML = '<strong class="who">' + input.username + ': </strong>' + input.body;
        chatElem.appendChild(para);

        scrollElem.scrollTop = scrollElem.scrollHeight;
    });

    await listen('init_channels', (event) => {
        const channelOptionList = document.getElementById("channels");
        const channelBox = document.getElementById("all-channel-boxes");

        for (const element of event.payload) {

            var channelOption = document.createElement("option");
            channelOption.innerHTML = element;
            channelOption.value = element;
            channelOptionList.appendChild(channelOption);
            
            var channelBoxInside = document.createElement("div");
            var innerChannel = document.createElement("div")
            channelBoxInside.className = "message-box"
            channelBoxInside.id = "channel-" + element;
            channelBoxInside.style.display = "none";
            innerChannel.className = "inner-channel";
            channelBoxInside.appendChild(innerChannel);
            channelBox.appendChild(channelBoxInside);

            invoke('request_history', { channel: element, amount: '50' });
        }
    });

    invoke('request_channels');
}

// Call the async function to start the initialization process
init().catch(console.error);
