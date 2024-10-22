
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
    console.log("Login clicked!");

    var post_data = {
        email : document.getElementById("LoginByEmailInputEmail").value,
    }
    api_post("login/email", post_data, login_callback);
}

function login_callback(data) {
    console.log("Arrived here:" + data)
}