
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
 * callback - function(return_code, json_data, callback_data)
 * callback_data - arbitraty data that is hand over to the callbackl function
 */
function api_v0_post(endpoint, tx_data, callback, callback_data) {
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
            response.json().then(json_data => {callback(response.status, json_data, callback_data)});
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



// ==========================================================================================
//                                    LiveInput
// HTML:
// <div class="LiveInput" id="MyId"><input ...><button>Save</button></div>
//
// JS:
// liveinput_init("MyId", prepare_save);
//
// function prepare_save(input_element) {
//     return {
//         api_endpoint: "change/my_value",
//         api_data: {value: input_element.value},
//     }
// }
//
// ==========================================================================================

function liveinput_init(id, prepare_api_call_function) {
    const e_div = document.getElementById(id);
    const e_inp = e_div.getElementsByTagName("input")[0];
    const e_btn = e_div.getElementsByTagName("button")[0];
    e_inp.addEventListener('input', liveinput_changed, {once:true});
    e_inp.LiveInputId = id;
    e_btn.LiveInputId = id;
    e_inp.LiveInputPrepareApiCallFunction = prepare_api_call_function;
}

function liveinput_changed(event) {
    const id = event.currentTarget.LiveInputId;
    const e_div = document.getElementById(id);
    const e_inp = e_div.getElementsByTagName("input")[0];
    const e_btn = e_div.getElementsByTagName("button")[0];

    e_inp.className = "Modified";
    e_btn.className = "Modified";
    e_btn.addEventListener("click", liveinput_save_clicked, {once:true});
}

function liveinput_save_clicked(event) {
    const id = event.currentTarget.LiveInputId;
    const e_div = document.getElementById(id);
    const e_inp = e_div.getElementsByTagName("input")[0];
    const e_btn = e_div.getElementsByTagName("button")[0];

    // lock elements
    e_inp.className = "Saving";
    e_btn.className = "Saving";

    // prepare api call
    const api_call_data = e_inp.LiveInputPrepareApiCallFunction(e_inp);
    api_v0_post(api_call_data.api_endpoint, api_call_data.api_data, liveinput_api_return, id);
}

function liveinput_api_return(return_code, json_data, id) {
    const e_div = document.getElementById(id);
    const e_inp = e_div.getElementsByTagName("input")[0];
    const e_btn = e_div.getElementsByTagName("button")[0];
    if (return_code === 200) {
        e_inp.className = "Saved";
        e_btn.className = "Saved";
        e_inp.addEventListener('change', liveinput_changed, {once:true});
    } else if (return_code === 500) {
        append_message_error(json_data.summary, json_data.description)
        console.log("Error: " + return_code);
        console.log(json_data);
        e_inp.className = "Failed";
        e_btn.className = "Failed";
    } else {
        console.log("Error: " + return_code);
        console.log(json_data);
        e_inp.className = "Failed";
        e_btn.className = "Failed";
    }
}