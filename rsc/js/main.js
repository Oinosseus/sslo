
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
