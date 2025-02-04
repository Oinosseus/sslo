
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