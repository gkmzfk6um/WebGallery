const pathRe = /\/store\/print\/([^\/]+)/
const match   = window.location.pathname.match(pathRe)

function oops()
{
    $(".body").addClass("oops");
}

function build(data)
{
    function sizechanged()
    {
        const index = $("#size").val();
        console.log(index)
        const price = data.variants[index].price.value
        $("#price").text(price+ "kr")
    }
    const displayname = data['image']['resource_data']['Image']['name'];
    $("#description").html(data['description'])
    $("#title").text(displayname);
    $("#image").css('background-color',data['image']['resource_data']['Image']['colour'])
    Object.entries(data.variants).forEach( (obj ,i)=> {
        var variant_name = obj[0];
        var variant = obj[1];
        var opt = $("<option>");
        $(opt).val(variant_name);
        $(opt).text(variant.width +"cm X " + variant.height + "cm");
        $("#size").append(opt)
    });
    $("#size").on('change', sizechanged)
    $("#close-print").click(() => window.location.href="/store#"+displayname )
    sizechanged();
    loadimage(data);
    $(".printItem").css("opacity","1")
    $('#addtocart').click( () => {
        var variantName =$('#size').val()
        var selectedVariant = data.variants[variantName];
        selectedVariant['name'] = variantName
        selectedVariant['signature'] = parseInt($('#signature').val());
        addItem(data['image']['id'], selectedVariant);
    });

}

function loadimage(data)
{
	const srcset=   "/" + data['thumbnails']['Huge']['path']   + " " + data['thumbnails']['Huge']  ['resource_data']['Thumbnail']['width'] + "w\n" 
	              + "/" + data['thumbnails']['Large']['path']  + " " + data['thumbnails']['Large'] ['resource_data']['Thumbnail']['width'] + "w\n" 
	              + "/" + data['thumbnails']['Medium']['path'] + " " + data['thumbnails']['Medium']['resource_data']['Thumbnail']['width'] + "w";
    const src ="/" + data['thumbnails']['Medium']['path'];
	let img = $('<img>')
	img.one('load',function() {
		$(this).css('visibility','visible');
		$(this).css('opacity', '1.0');
	})
	img.attr('srcset',srcset);
	img.attr('src',src);
    $("#image").append(img)
}

function resize()
{


}

function initPrint()
{
    if (match)
    {
        const name = match[1];
        const apiPath = '/api/print/name/'+name;
        $.ajax( {
            url: apiPath,
            dataType: "json",
            success: (data) => build(data),
            error: () => oops()
        });
    }
    else 
    {
        oops();
    }

}

$(document).ready(() => initPrint());