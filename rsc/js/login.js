function tabSelectByIndex(active_index) {

    const buttons = [
        document.getElementById("LoginButtonLoginPassword"),
        document.getElementById("LoginButtonLoginEmail"),
        document.getElementById("LoginButtonLoginSteam"),
        document.getElementById("RegisterButton"),
    ];

    for (let i = 0; i < buttons.length; i++) {
        if (i == active_index) {
            buttons[i].classList.add("ActiveButton");
        } else {
            buttons[i].classList.remove("ActiveButton");
        }
    }

    const tabs = [
        document.getElementById("TabLoginPassword"),
        document.getElementById("TabLoginEmail"),
        document.getElementById("TabLoginSteam"),
        document.getElementById("TabRegisterSsloForm"),
    ]


    for (let i = 0; i < tabs.length; i++) {
        if (i == active_index) {
            tabs[i].classList.add("ActiveTab");
        } else {
            tabs[i].classList.remove("ActiveTab");
        }
    }
}
