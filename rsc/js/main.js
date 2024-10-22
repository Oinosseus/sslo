
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

/** Make a POST call to the v0 API
 * endpoint - the relative path to the REST API endpoint
 * tx_data - an object that can be json parsed
 * callback - function(return_code, json_data)
 */
function api_v0_post(endpoint, tx_data, callback) {
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
            response.json().then(json_data => {callback(response.status, json_data)});
        })
        .catch(error => {
            console.log("API error: " + error);
        })
}



function remove_quotes(text) {
    if (text.charAt(0) == "\"" && text.charAt(text.length-1) == "\"") {
        text = text.slice(1, text.length-1);
    }
    return text;
}


function append_message(css_class, title, message) {
    title = remove_quotes(title);
    message = remove_quotes(message);

    // create heading
    var new_msg_hdr = document.createElement("div");
    new_msg_hdr.classList.add("Label");
    var new_msg_head_txt = document.createTextNode(title);
    new_msg_hdr.appendChild(new_msg_head_txt);

    // create message
    var new_msg = document.createElement("div");
    new_msg.classList.add("Message");
    var new_msg_text = document.createTextNode(message)
    new_msg.appendChild(new_msg_text);

    // compile message
    var new_sub_msg = document.createElement("div");
    new_sub_msg.classList.add(css_class);
    new_sub_msg.appendChild(new_msg_hdr);
    new_sub_msg.appendChild(new_msg);

    // append to document
    let new_tag = document.getElementsByTagName("messages")[0];
    new_tag.appendChild(new_sub_msg);
}

function append_message_error(title, message) {
    append_message("MessageError", title, message);
}

function append_message_warning(title, message) {
    append_message("MessageWarning", title, message);
}

function append_message_success(title, message) {
    append_message("MessageSuccess", title, message);
}
