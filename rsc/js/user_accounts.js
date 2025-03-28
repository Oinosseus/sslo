function disable_all_elements(disable) {
    // let switch_log_reg = document.getElementById("SwitchLoginRegister").getElementsByTagName("input")[0];
    // switch_log_reg.disabled = disable;

    // if (switch_log_reg.checked) {
    //     document.getElementById("WithPasswordId").disabled = true;
    //     document.getElementById("WithPasswordPassword").disabled = true;
    //     document.getElementById("WithPasswordButton").disabled = true;
    // } else {
    //     document.getElementById("WithPasswordId").disabled = disable;
    //     document.getElementById("WithPasswordPassword").disabled = disable;
    //     document.getElementById("WithPasswordButton").disabled = disable;
    // }
    //
    // document.getElementById("WithEmailEmail").disabled = disable;
    // document.getElementById("WithEmailButton").disabled = disable;
    //
    // document.getElementById("WithSteamButton").disabled = disable;

    if (disable) {
        document.body.style.cursor = "wait";
    } else {
        document.body.style.cursor = "default";
    }
    busy_spinner(disable);
}

function tabSelectByIndex(active_index) {

    // definition of buttons
    const buttons = [
        document.getElementById("AccountTypeButtonPassword"),
        document.getElementById("AccountTypeButtonEmail"),
        document.getElementById("AccountTypeButtonSteam"),
        document.getElementById("AccountTypeButtonDiscord"),
    ];

    // definition of tabs
    const tabs = [
        document.getElementById("AccountTabPassword"),
        document.getElementById("AccountTabEmail"),
        document.getElementById("AccountTabSteam"),
        document.getElementById("AccountTabDiscord"),
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
            tabs[i].classList.add("TabActive");
            tabs[i].classList.remove("TabInActive");
        } else {
            tabs[i].classList.add("TabInActive");
            tabs[i].classList.remove("TabActive");
        }
    }
}


document.addEventListener('DOMContentLoaded', function () {
    var live_input = document.getElementById("AccountTabPassword");
    liveinput_init("AccountTabPassword", prepare_password_save)
})

function prepare_password_save(input_elements) {

    // collect data
    var pwd_current = "";
    var pwd_new1 = "";
    var pwd_new2 = "";
    for (i = 0; i < input_elements.length; i++) {
        inp = input_elements[i];
        switch (inp.name) {
            case "PasswordCurrent":
                pwd_current = inp.value;
                break;
            case "PasswordNew1":
                pwd_new1 = inp.value;
                break;
            case "PasswordNew2":
                pwd_new2 = inp.value;
                break;
        }
    }

    // validate new password
    if (pwd_new1 !== pwd_new2) {
        append_message_error("Password Mismatch", "Thew new password and repeated password are not equal!")
        return null;
    }

    return {
        api_endpoint:"user/set_password",
        api_data:{old_password: pwd_current, new_password: pwd_new1},
    };
}

function handler_button_delete_email(email) {
    let tx_data= { email: email };
    api_v0("DELETE", "user/account/email", tx_data, handler_button_email_callback);
    console.log("DELETE " + email);
    disable_all_elements(true);
}

function handler_button_add_email() {
    let inp_eml = document.getElementById("AddEmail");

    // check valid email
    if (!is_valid_email(inp_eml.value)) {
        append_message_error("Invalid Email", "The entered new email '" + inp_eml.value + "' is invalid!");
        return;
    }

    // send request
    let tx_data= { email: inp_eml.value };
    api_v0("PUT", "user/account/email", tx_data, handler_button_email_callback);
    disable_all_elements(true);
}

function handler_button_email_callback(status, data) {
    if (status == 204) {
        location.reload();
    } else if (status == 400) {
        disable_all_elements(false);
        append_message_error(data.summary, data.description);
    } else if (status == 403) {
        disable_all_elements(false);
        append_message_error(data.summary, data.description);
    } else if (status == 500) {
        disable_all_elements(false);
        append_message_error(data.summary, data.description);
    } else {
        disable_all_elements(false);
        append_message_error("Unexpected Error", data);
    }
}