// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { sendNotification } = window.__TAURI__.notification;
const { appWindow } = window.__TAURI__.window;

function sendMessage(body) {
    const channelList = document.getElementById("channels");
    const dmList = document.getElementById("dms")
    if (window.messageMode == "public") {
        invoke('send_message', { message: body, channel: channelList.options[channelList.selectedIndex].value, visibility: 'public' })
    }
    else {
        invoke('send_message', {message: body, target: dmList.options[dmList.selectedIndex].value, visibility: 'dm'})
    }
    
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

function dmChange() {
    const dmHolder = document.getElementById("all-dm-boxes");
    const dmList = document.getElementById("dms");

    const channels = dmHolder.getElementsByClassName("message-box");
    for (const channelBox of channels) {
        channelBox.style.display = "none";
    };

    const activeDm = document.getElementById("dm-" + dmList.options[dmList.selectedIndex].value);
    activeDm.style.display = "block";
    activeDm.scrollTop = activeDm.scrollHeight;
}

function setTab(tab) {
    const channelTab = document.getElementById("channels-tab");
    const dmTab = document.getElementById("dms-tab");
    let channelsActive = tab == "channels"
    let activeTab = channelsActive ? channelTab : dmTab;
    let inactiveTab = channelsActive ? dmTab : channelTab;
    window.messageMode = channelsActive ? "public" : "dm";

    activeTab.style.display = "block";
    inactiveTab.style.display = "none";
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
        let is_dm = input.channel.startsWith('DM_');
        let username = window.localStorage.getItem("username");
        let dm_name = input.channel.replace("DM_", "").replace(username, "").replace("_", "");
        let id = is_dm ? "dm-" + dm_name : "channel-" + input.channel;
        console.log(id);

        const scrollElem = document.getElementById(id);
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

            invoke('request_history', { channel: element, amount: '50', visibility: 'public' });
        }
    });

    await listen('init_users', (event) => {
        const userOptionList = document.getElementById("dms");
        const dmBox = document.getElementById("all-dm-boxes");
        for (const username of event.payload) {

            var userOption = document.createElement("option");
            userOption.innerHTML = username;
            userOption.value = username;
            userOptionList.appendChild(userOption);
            
            var dmBoxInside = document.createElement("div");
            var innerChannel = document.createElement("div")
            dmBoxInside.className = "message-box"
            dmBoxInside.id = "dm-" + username;
            dmBoxInside.style.display = "none";
            innerChannel.className = "inner-channel";
            dmBoxInside.appendChild(innerChannel);
            dmBox.appendChild(dmBoxInside);

            invoke('request_history', { target: username, amount: '50', visibility: 'dm' });
        }
    });

    invoke('request_channels');
    invoke('request_users');
    window.messageMode = "public"
}

// Call the async function to start the initialization process
init().catch(console.error);
