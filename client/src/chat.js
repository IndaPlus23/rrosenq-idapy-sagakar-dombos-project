// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

function sendMessage(body, who) {
    invoke('send_message', { message: body })
}

// Define an async function to use await
async function init() {
    invoke('greet', { name: 'World' })
        // `invoke` returns a Promise
        .then((response) => {
            window.header.innerHTML = response;
        });

    // Now you can use await within an async function
    await listen('recieve_message', (event) => {
        console.log("message received: " + event);
        let input = event.payload;

        const chatElem = document.getElementById("inner-messages");
        const scrollElem = document.getElementById("messages");

        var para = document.createElement("p");
        para.innerHTML = '<strong class="who">' + "you" + ': </strong>' + input;
        chatElem.appendChild(para); // to be moved to listen event

        scrollElem.scrollTop = scrollElem.scrollHeight;
    });
}

// Call the async function to start the initialization process
init().catch(console.error);
