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
