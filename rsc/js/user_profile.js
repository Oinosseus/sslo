document.addEventListener('DOMContentLoaded', function () {
    liveinput_init("ProfileUserName", username_prepare_save);
    liveinput_init("ProfileUserPassword", password_prepare_save);
})

function username_prepare_save(input_elements) {
    return {
        api_endpoint:"user/set_name",
        api_data:{name: input_elements[0].value},
    };
}


function password_prepare_save(input_elements) {

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
