import React from "react";
import App from "./App";

function LoginPage () {

    var userName, passWord = prompt("Enter name", "Enter Password")

    return (
        <div>
            <App/>
        </div>
    )
}

export default LoginPage;