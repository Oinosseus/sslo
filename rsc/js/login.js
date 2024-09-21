function tabSelectSsloLogin() {

    // activate buttons
    document.getElementById("LoginSsloButton").classList.add("ActiveButton");
    document.getElementById("RegisterSsloButton").classList.remove("ActiveButton");

    // show/hide tabs
    document.getElementById("TabLoginSsloForm").classList.add("ActiveTab");
    document.getElementById("TabRegisterSsloForm").classList.remove("ActiveTab");
}

function tabSelectSlloRegister() {

    // activate buttons
    document.getElementById("LoginSsloButton").classList.remove("ActiveButton");
    document.getElementById("RegisterSsloButton").classList.add("ActiveButton");

    // show/hide tabs
    document.getElementById("TabLoginSsloForm").classList.remove("ActiveTab");
    document.getElementById("TabRegisterSsloForm").classList.add("ActiveTab");
}
