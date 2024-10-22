
document.addEventListener('DOMContentLoaded', function () {
});


function navbarDropdown(element) {

    let element_was_active = element.parentElement.classList.contains("NavbarDropdownActive");

    // close all other dropdowns
    var x = document.getElementsByClassName('NavbarDropdown');
    for (var i = 0; i < x.length; i++) {
        if (x[i] != element) {
            x[i].classList.remove("NavbarDropdownActive");
        }
    }

    // assign active class to element parent
    if (!element_was_active) {
        element.parentElement.classList.add("NavbarDropdownActive");
    }
}


function api_post(endpoint, tx_data, callback_200) {
    let tx_data_string = JSON.stringify(tx_data);
    fetch("/api/v0/" + endpoint, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "Content-Length": tx_data_string.length,
        },
        body: tx_data_string,
    })
        .then(response => {
            console.log("API response: " + response.data);
            switch (response.status) {
                case 200:
                    response.json().then(data => {callback_200(data)});
                    break
                case 400:
                    response.json().then(data => {
                        console.log("API Error: " + data.title + "\n" + data.description);
                        append_message_error(data.title, data.description);
                    });
                    break;
                case 500:
                    response.json().then(data => {
                        console.log("API Error: " + data.title + "\n" + data.description);
                        append_message_error(data.title, data.description);
                    });
                    break;
                default:
                    console.log("API Request failed: " + response.status);
                    break;
            }
        })
        .catch(error => {
            console.log("API error: " + error);
        })
}