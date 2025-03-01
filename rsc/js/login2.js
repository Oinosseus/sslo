function disable_all_elements(disable) {
    let switch_log_reg = document.getElementById("SwitchLoginRegister").getElementsByTagName("input")[0];
    switch_log_reg.disabled = disable;

    if (switch_log_reg.checked) {
        document.getElementById("WithPasswordId").disabled = true;
        document.getElementById("WithPasswordPassword").disabled = true;
        document.getElementById("WithPasswordButton").disabled = true;
    } else {
        document.getElementById("WithPasswordId").disabled = disable;
        document.getElementById("WithPasswordPassword").disabled = disable;
        document.getElementById("WithPasswordButton").disabled = disable;
    }

    document.getElementById("WithEmailEmail").disabled = disable;
    document.getElementById("WithEmailButton").disabled = disable;

    document.getElementById("WithSteamButton").disabled = disable;
}


// Switching Login/Register functionality
document.addEventListener('DOMContentLoaded', function () {
    let switch_log_reg = document.getElementById("SwitchLoginRegister").getElementsByTagName("input")[0];

    function add_classes(class_name) {
        document.getElementById("LabelLogin").classList.add(class_name);
        document.getElementById("LabelRegister").classList.add(class_name);

        document.getElementById("WithPasswordId").classList.add(class_name);
        document.getElementById("WithPasswordPassword").classList.add(class_name);
        document.getElementById("WithPasswordButton").classList.add(class_name);

        document.getElementById("WithEmailEmail").classList.add(class_name);
        document.getElementById("WithEmailButton").classList.add(class_name);

        document.getElementById("WithSteamButton").classList.add(class_name);
    }

    function rm_classes(class_name) {
        document.getElementById("LabelLogin").classList.remove(class_name);
        document.getElementById("LabelRegister").classList.remove(class_name);

        document.getElementById("WithPasswordId").classList.remove(class_name);
        document.getElementById("WithPasswordPassword").classList.remove(class_name);
        document.getElementById("WithPasswordButton").classList.remove(class_name);

        document.getElementById("WithEmailEmail").classList.remove(class_name);
        document.getElementById("WithEmailButton").classList.remove(class_name);

        document.getElementById("WithSteamButton").classList.remove(class_name);
    }

    function switch_log_reg_handler() {
        if (switch_log_reg.checked) {
            add_classes("SwitchedToRegister");
            rm_classes("SwitchedToLogin");
            document.getElementById("WithPasswordId").disabled = true;
            document.getElementById("WithPasswordPassword").disabled = true;
            document.getElementById("WithPasswordButton").disabled = true;
        } else {
            add_classes("SwitchedToLogin");
            rm_classes("SwitchedToRegister");
            document.getElementById("WithPasswordId").disabled = false;
            document.getElementById("WithPasswordPassword").disabled = false;
            document.getElementById("WithPasswordButton").disabled = false;
        }
    }

    switch_log_reg.addEventListener("change", switch_log_reg_handler);
    switch_log_reg_handler();
})

// With Password button handling
document.addEventListener('DOMContentLoaded', function () {
    let btn = document.getElementById("WithPasswordButton");
    btn.addEventListener("click", function() {

        // disable UI
        disable_all_elements(true);

        // prepare request data
        let request_data = {};
        request_data.identification = document.getElementById("WithPasswordId").value;
        request_data.password = document.getElementById("WithPasswordPassword").value;

        // call api
        api_v0_post("login/password", request_data, password_login_api_callback)
    })
})

function password_login_api_callback(status, data) {
    if (status == 200) {
        append_message_success("OK", "Successful login");
        window.location.href = "/";
    } else {
        append_message_error("ERROR", "Login failed");
        disable_all_elements(false);
    }
}


// With Email button handling
document.addEventListener('DOMContentLoaded', function () {
    let btn = document.getElementById("WithEmailButton");
    btn.addEventListener("click", function() {
        let switch_log_reg = document.getElementById("SwitchLoginRegister").getElementsByTagName("input")[0];
        let inp_email = document.getElementById("WithEmailEmail");

        // disable UI
        disable_all_elements(true);

        // verify email address
        if (!is_valid_email(inp_email.value)) {
            append_message_error("Invalid Email", "Please enter valid email address!");
            disable_all_elements(false);
            return;
        }

        // register new email
        if (switch_log_reg.checked) {
            window.location.href = window.location.origin + "/html/login_email_create/" + inp_email.value;
        } else {
            window.location.href = window.location.origin + "/html/login_email_existing/" + inp_email.value;
        }
    })
})