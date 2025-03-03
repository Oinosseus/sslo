document.addEventListener('DOMContentLoaded', function () {
    var form = document.getElementById("UserSettingsForm");
    if (form) {
        form.addEventListener('submit', function (event) {

            const form_data = new FormData(form);
            var request_data = new Object();

            // name
            var form_data_new_name = form_data.get("new_name");
            if (form_data_new_name.length > 0) {
                request_data.new_name = form_data_new_name;
            }

            // old password
            var form_data_old_password = form_data.get("old_password");
            if (form_data_old_password.length > 0) {
                request_data.old_password = form_data_old_password;
            }

            // new password
            var form_data_new_password1 = form_data.get("new_password1");
            var form_data_new_password2 = form_data.get("new_password2");
            if (form_data_new_password1.length > 0 || form_data_new_password2.length > 0) {
                if (form_data_new_password1 !== form_data_new_password2) {
                    append_message_error("Failure", "New password does not match to verify password!");
                    event.preventDefault();
                    return false;
                } else {
                    request_data.new_password = form_data_new_password1;
                }
            }

            // disable further changes (must be done after getting form data)
            for (var i = 0, len = form.elements.length; i < len; ++i) {
                form.elements[i].setAttribute("disabled", "true");
            }

            // call api
            api_v0_post("user/update_settings", request_data, api_callback)

            // prevent default form submission
            event.preventDefault();
            return false;
        })
    }
});


function api_callback(status, data) {
    if (status == 200) {
        append_message_success("OK", "Settings changed successfully.");
        location.reload();

    } else if (status == 500) {
        append_message_error(data.summary, data.description);

    } else {
        append_message_error("ERROR", "Failed to change settings: " + data);

    }

    // re-enable form
    var form = document.getElementById("UserSettingsForm");
    for (var i = 0, len = form.elements.length; i < len; ++i) {
        form.elements[i].setAttribute("disabled", "true");
    }
}
