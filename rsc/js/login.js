function tabSelectByIndex(active_index) {

    // definition of buttons
    const buttons = [
        document.getElementById("LoginButtonLoginPassword"),
        document.getElementById("LoginButtonLoginEmail"),
        document.getElementById("LoginButtonLoginSteam"),
    ];

    // definition of tabs
    const tabs = [
        document.getElementById("TabLoginPassword"),
        document.getElementById("TabLoginEmail"),
        document.getElementById("TabLoginSteam"),
    ]

    // activation of buttons
    for (let i = 0; i < buttons.length; i++) {
        if (i == active_index) {
            buttons[i].classList.add("ActiveButton");
        } else {
            buttons[i].classList.remove("ActiveButton");
        }
    }

    // activation of tabs
    for (let i = 0; i < tabs.length; i++) {
        if (i == active_index) {
            tabs[i].classList.add("ActiveTab");
        } else {
            tabs[i].classList.remove("ActiveTab");
        }
    }
}


function buttonLoginEmail() {

    // deactivate login button
    document.getElementById("LoginByEmailButton").setAttribute("disabled", "true");

    // ge form data
    var post_data = {
        email : document.getElementById("LoginByEmailInputEmail").value,
    }

    // send API request
    api_v0_post("login/email", post_data, login_callback);
}


function login_callback(response_code, json_data) {
    if (response_code == 200) {
        append_message_success("An email with login data should be sent.",
            "When email already exists, a new token was sent. When a token was already sent, nothing happens.");
    } else {
        append_message_error("Error",
            "Something went wrong. Please try again.");
    }
}