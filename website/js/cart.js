var cart = {
    version: 0,
    items: {

    }
}



const cookieName = 'gallery-cart'
const cookieProperty = 
{
    path: "/store/",
    SameSite: true
}
function initClick()
{
    var cart = $('#cart');
    var cartClose = $('#cart-close')
    var className = 'cart-open'
    var open  = () =>
    { 
        console.log('open')
        cart.addClass(className);
        cart.off('click')
    };


    cart.click(open)
    cartClose.click( (e) => {
        console.log('close')
       cart.removeClass(className)
       e.stopPropagation();
       cart.click(open)
    }) 
}

function init()
{
    initClick();
    var cookie = Cookies.get(cookieName,cookieProperty)
    if (typeof cookie === 'undefined'){

    }
    else
    {
       const obj = JSON.parse(atob(cookie))
       if (obj.version < cart.version)
       {
           console.log('Cart cookie not supported!')
       }
       else
       {
           cart = obj;
       }
    }
    buildUI();
}

function each(obj, fn) { Object.keys(obj).forEach(key => fn(key, obj[key])); }

function buildUI()
{

    var items =0 ;
    var $cartItems = $('#cartitems')
    each(cart.items, (key,item) => {
        const b64Id = btoa(key)
        var $uiItem = $('#'+b64Id)
        if (item.quantity == 0)
        {
            $uiItem.addClass('removed');
        }
        else
        {
            items += item.quantity;
            if ($uiItem.length == 0)
            {
                $uiItem = $($('#cartitemtemplate').prop('content')).clone();
                $cartItems.append($uiItem);
                $uiItem = $cartItems.find('*:last-child')
                $uiItem.attr('id',b64Id)
                $uiItem.addClass('added')
            }
            $uiItem.find(".quantity").text(item.quantity);
        }
     });
     $('#quantity').text(items);
}


function updateCookie()
{
    var cookieContent = btoa(JSON.stringify(cart));
    Cookies.set(cookieName,cookieContent,cookieContent);
}

function update()
{
    buildUI();
    updateCookie();

}

function identifier(id,variant)
{
    return id + variant.height + "h"+variant.width+"w"+variant.signature + "s";
}

function addItem(id,variant)
{
    const idString = identifier(id,variant);
    if (idString in cart.items)
    {
        console.log('Add addition item' + idString)
        cart.items[idString].quantity += 1;
    }
    else {
        console.log('Add item' + idString)
        cart.items[idString] = {
            quantity: 1,
            variant: variant,
            id: id
        };
    }
    update();
}

function removeItem(id,variant,eraseAll)
{
    const idString = identifier(id,variant);
    if (idString in cart.items)
    {
        var toBeRemoved = 1;
        if (eraseAll)
        {
            toBeRemoved = cart.items[idString];
        }
        cart.items[idString].quantity -= 1;
        if (cart.items[idString].quantity < 0 )
        {
            cart.items[idString].quantity = 0;
        }
    }
    update();
}


init();