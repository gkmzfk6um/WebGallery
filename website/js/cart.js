const cartTemplate = {
    version: 0,
    items: {

    }
}


var cart = cartTemplate;
var uiIds = {
    next: 0,
    forward: {},
    backward: {},
}


var cartInfo = {

}

function findVariantInfo(cartItem)
{
    const info = cartInfo[cartItem.id];
    var foundVariant;
    info.variants.every(function(variant)
    {
        if (cartItem.variant.width == variant.width && cartItem.variant.height == variant.height)
        {
            foundVariant = variant;
            return false;
        }
        return true;
    })
    return foundVariant;
}


function updateCartInfo(cb)  {

    function response(obj)
    {
        each(obj.success,function(k,v)
        {
            cartInfo[k] = v;
        });
        var itemsRemoved = false;

        each(cart.items,function(k,v){
            if (!(v.id in cartInfo))
            {
                console.log('Removing ' + k +' from cart, no such item found')
                v.quantity=0;
                itemsRemoved=true;
            }
            if (!findVariantInfo(v))
            {
                console.log('Removing ' + k +' from cart, no such item found')
                v.quantity=0;
                itemsRemoved=true;
            }
        });

        if (Object.keys(obj.failed).length > 0)
        {
            itemsRemoved=true;
        }
        if (itemsRemoved)
        {
            alert('Some items in the cart could not be found and has been removed.');
        }

        cb();
    }
    var id2req ={};
    each(cart.items,function(k,v)
    {
        if (!(v.id in cartInfo)){
            id2req[v.id]=true;
        }
    })

    id2req = Object.keys(id2req)
    if (id2req.length > 0)
    {
        $.ajax({
        type: 'POST',
        url: '/api/store/info',
        data: JSON.stringify (id2req),
        success: response,
        error: () => alert('Failed to communicate with sales server, please try again later. If the issue persist please contact me and I\'ll assist you'),
        contentType: "application/json",
        dataType: 'json'});
    }
    else
    {
        cb();
    }
}

function checkout()
{
    function checkoutResponse(data)
    {
        console.log(data)
        window.location.assign(data);
    }

    if (Object.keys(cart.items).length > 0 && !$checkoutButton.hasClass('disabled'))
    {
        $.ajax({
        type: 'POST',
        url: '/api/store/checkout',
        data: JSON.stringify (cart),
        success: checkoutResponse,
        error: () => alert('Failed to communicate with sales server, please try again later. If the issue persist please contact me and I\'ll assist you'),
        contentType: "application/json",
        dataType: 'text'});
    }
}

function allocateId(identifier)
{
    const id = "item"+uiIds.next;
    uiIds.forward[identifier]=id;
    uiIds.backward[id]=identifier;
    uiIds.next += 1;
    return id;
} 

function getOrAllocateId(identifier)
{
    if (identifier in uiIds.forward)
    {
        return uiIds.forward[identifier]
    }
    else
    {
        return allocateId(identifier);
    }
}

function freeId(identifier)
{
    delete uiIds.backward[uiIds.forward[identifier]]
    delete uiIds.forward[identifier]
}



const storageName = 'gallery-cart'

const $cart = $('#cart');
const $cartClose = $('#cart-close')
const cartOpenclass = 'cart-open'
const $checkoutButton = $('#checkout')

var openCart  = () =>
{ 
    $cart.addClass(cartOpenclass);
    $cart.off('click')
};

function initClick()
{
    $cart.click(openCart)
    $cartClose.click( (e) => {
       $cart.removeClass(cartOpenclass)
       e.stopPropagation();
       $cart.click(openCart)
    }) 
    $checkoutButton.click(checkout);
}

function init()
{
    initClick();
    var storageContent = localStorage.getItem(storageName)
    try 
    {
        if (typeof storageContent === 'undefined'){

        }
        else
        {
           const obj = JSON.parse(atob(storageContent))
           if (obj.version < cart.version)
           {
               console.log('Local storage cart version not supported')
               localStorage.removeItem(storageName);
           }
           else
           {
                    cart = obj;
           }
        }
        updateCartInfo( () => {
            buildUI();
        });
    }
    catch(err)
    {
        localStorage.removeItem(storageName);
        cart = cartTemplate;
        updateCartInfo( () => {
            buildUI();
        });
    }
}

function clickCartItem(e)
{
    var $button = $(e.currentTarget);
    var $item   = $button.parents('.cartitem');
    var id      = uiIds.backward[$item.attr('id')]
    var item    = cart.items[id];
    
    if ($button.hasClass('close-container'))
    {
        removeItem(item.id,item.variant,true);
    }
    else if ($button.hasClass('minusbutton'))
    {
        removeItem(item.id,item.variant,false);
    }
    else if ($button.hasClass('plusbutton'))
    {
        addItem(item.id,item.variant);
    }
}

function each(obj, fn) { Object.keys(obj).forEach(key => fn(key, obj[key])); }

function buildUI()
{
    function variant2desc(variant){
        const  signatureOptions = 
        [
            'Sign front',
            'Sign rear',
            "Don't sign"
        ];
        return `${variant.width}cm x ${variant.height} cm, ${signatureOptions[variant.signature]}`
    }

    var items =0 ;
    var totalPrice = 0;
    var $cartItems = $('#cartitems')
    $cartItems.find(".removed").remove();
    each(cart.items, function(key,item) {
        var uiId = getOrAllocateId(key);
        var $uiItem = $('#'+uiId)
        if (item.quantity == 0)
        {
            $uiItem.removeClass('added');
            $uiItem.addClass('removed');
            freeId(key)
        }
        else
        {
            items += item.quantity;
            if ($uiItem.length == 0)
            {
                $uiItem = $($('#cartitemtemplate').html());
                $uiItem.attr('id',uiId)
                $uiItem.find(".close-container").click(clickCartItem)
                $uiItem.find(".minusbutton").click(clickCartItem)
                $uiItem.find(".plusbutton").click(clickCartItem)
                $cartItems.append($uiItem);
                console.log(cartInfo[item.id].name)
                console.log($uiItem.find(".itemname"))
                var $itemName = $uiItem.find(".itemname")
                $itemName.text(cartInfo[item.id].name);
                $itemName.attr('href',`/store/print/${item.id}`)
                $uiItem.find(".itemoption").text(variant2desc(item.variant))
                $uiItem.addClass('added')
            }
            var variant = findVariantInfo(item);
            var price = variant.price * item.quantity;
            $uiItem.find(".quantity").text(item.quantity);
            $uiItem.find(".itemprice").text(price+'kr');
            totalPrice+=price

        }
     });

     var $quantity = $('#quantity');
     var $price    = $('#totalprice')
     $quantity.text(items);
     $price.text(`${totalPrice}`);

     if (items > 0)
     {
        $quantity.addClass('visible');
        $checkoutButton.removeClass('disabled');
     }
     else
     {
        $quantity.removeClass('visible');
        $checkoutButton.addClass('disabled');
     }
}


function updateLocalStorage()
{
    var storageContent = btoa(JSON.stringify(cart));
    localStorage.setItem(storageName,storageContent);
}

function update()
{
    updateCartInfo( () => {
        buildUI();
        each(cart.items, (key,val) => val.quantity == 0 ? delete cart.items[key] : undefined);
        if (!$cart.hasClass(cartOpenclass))
        {
            openCart();
        }

        updateLocalStorage();
    });

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
        cart.items[idString].quantity += 1;
    }
    else {
        cart.items[idString] = {
            quantity: 1,
            variant: {
                width: variant.width,
                height: variant.height,
                signature: variant.signature
            },
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
            toBeRemoved = cart.items[idString].quantity;
        }
        cart.items[idString].quantity -= toBeRemoved;
        if (cart.items[idString].quantity < 0 )
        {
            cart.items[idString].quantity = 0;
        }
    }
    update();
}


init();