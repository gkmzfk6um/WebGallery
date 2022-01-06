
function menu()
{
    const menu = $(".slideout-menu");
    const hide = "slideout-menu-hide";
    const show = "slideout-menu-show";
    const showAlways = "slideout-menu-visible";
    const bodyDiv = $("div.body");
    const body    = $('body');
    const logo    = $('#site-logo');
    $('#menu-close').click( function() {
        if (!menu.hasClass(hide))
        {
            menu.removeClass (show);
            menu.addClass (hide);
        }
    });
    logo.click( function() {
        if (!menu.hasClass(show))
        {
            menu.removeClass(hide);
            menu.addClass(show);
        }
    });

    function checkMenu()
    {
        const bodyRect = bodyDiv.get(0).getBoundingClientRect();
        const menuRect = menu.get(0).getBoundingClientRect();
        const logoRect = logo.get(0).getBoundingClientRect();
        const right = (menuRect.right > logoRect.right) ? menuRect.right : logoRect.right;
        if (bodyRect.left > right+10)
        {
            if (!body.hasClass(showAlways))
            {
                body.addClass(showAlways);
            }
        }
        else
        {
            if (body.hasClass(showAlways))
            {
                body.removeClass(showAlways);
            }
        }
    }

    var timer;
    $(window).resize(function() {
    	clearTimeout(timer);
    	timer = setTimeout(checkMenu,100)
    });
    checkMenu();
}

$(document).ready(() => menu());