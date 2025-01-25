document.addEventListener('DOMContentLoaded', function () {
    liveinput_init("ProfileUserName", username_prepare_save);
})

function username_prepare_save(input_element) {
    return {
        api_endpoint:"user/set_name",
        api_data:{name: input_element.value},
    };
}
