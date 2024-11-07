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

// This code uses REST API as email/password login
// comment out to use http form processing
document.addEventListener('DOMContentLoaded', function () {
    var form = document.getElementById("TabLoginPassword");
    form.addEventListener('submit', function (event) {

        // create json request data from form data
        const form_data = new FormData(form);
        var request_data = new Object();
        request_data.email = form_data.get("email");
        request_data.password = form_data.get("password");

        // disable further changes (must be done after getting form data)
        for (var i = 0, len = form.elements.length; i < len; ++i) {
            form.elements[i].setAttribute("disabled", "true");
        }

        // call api
        api_v0_post("login/email", request_data, api_callback)

        // prevent default form submission
        event.preventDefault();
        return false;
    })
})



function api_callback(status, data) {
    if (status == 200) {
        append_message_success("OK", "Successful login");
        window.location.href = "/";

    } else {
        append_message_error("ERROR", "Login failed");
    }

    // re-enable form
    var form = document.getElementById("TabLoginPassword");
    for (var i = 0, len = form.elements.length; i < len; ++i) {
        form.elements[i].setAttribute("disabled", "true");
    }
}
