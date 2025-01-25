document.addEventListener('DOMContentLoaded', function () {
    document.getElementById("InputUserName").addEventListener('change', username_changed, {once:true});
})

function username_changed() {
    const btn = document.getElementById("InputUserNameSaveButton");
    btn.style.display = 'inline';
    btn.addEventListener("click", username_save_clicked, {once:true});
}

function username_save_clicked() {
    const inp = document.getElementById("InputUserName");
    const btn = document.getElementById("InputUserNameSaveButton");

    // lock elements
    inp.setAttribute("disabled", "true");
    btn.style.display = 'none';

    // call API
    api_v0_post("user/set_name", {name: inp.value}, username_saved);
}

function username_saved(return_code, json_data) {
    if (return_code == 200) {
        const inp = document.getElementById("InputUserName");
        inp.removeAttribute("disabled");
        inp.addEventListener('change', username_changed, {once:true});
    } else {
        console.log("Error: " + return_code);
        console.log(json_data);
    }
}