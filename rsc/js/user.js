document.addEventListener('DOMContentLoaded', function () {
    var form = document.getElementById("UserSettingsForm");
    form.addEventListener('submit', function (event) {

        // call api
        const form_data = new FormData(form);
        var request_data = new Object();
        request_data.new_name = form_data.get("new_name");
        api_v0_post("user/set_name", request_data, api_callback)

        // disable further changes
        for (var i = 0, len = form.elements.length; i < len; ++i) {
            form.elements[i].setAttribute("disabled", "true");
        }

        // prevent default form submission
        event.preventDefault();
        return false;
    })
});


function api_callback(status, data) {
    if (status == 200) {
        append_message_success("OK", "Name changed successfully.");
        location.reload();

    } else {
        append_message_error("ERROR", "Failed to change name: " + data);

        // re-enable form
        var form = document.getElementById("UserSettingsForm");
        for (var i = 0, len = form.elements.length; i < len; ++i) {
            form.elements[i].setAttribute("disabled", "true");
        }
    }
}
