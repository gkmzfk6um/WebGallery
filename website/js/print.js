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
        const price = data.variants[index].price.value
        $("#price").text(price+ "kr")
    }

    $("#description").text(data['description'])
    $("#title").text(data['data']['displayname'])
    $("#image").css('background-color',data['data']['colour'])
    $("#viewlink").attr("href","/"+data['data']['view'].replace(/\.html$/,"") + "?src=print&stay")
    data.variants.forEach( (variant ,i)=> {
        var opt = $("<option>");
        $(opt).val(i);
        $(opt).text(variant.width +"cm X " + variant.height + "cm");
        $("#size").append(opt)
    });
    $("#size").on('change', sizechanged)
    $("#close-print").click(() => window.location.pathname="/store" )
    sizechanged();
    loadimage(data);
    $(".printItem").css("opacity","1")
    $('#addtocart').click( () => {
        var selectedVariant = data.variants[$('#size').val()];
        selectedVariant['signature'] = $('#signature').val();
        addItem(data['data']['dropbox']['id'], selectedVariant);
    });

}

function loadimage(data)
{
	const srcset=   "/" + data['data']['huge']['path']   + " " + data['data']['huge']['width']   + "w\n" 
	              + "/" + data['data']['large']['path']  + " " + data['data']['large']['width']  + "w\n" 
	              + "/" + data['data']['medium']['path'] + " " + data['data']['medium']['width'] + "w";
    const src ="/"+ data['data']['medium']['path'];
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
        const id = match[1];
        const apiPath = '/api/print/'+id;
        console.log(match)
        console.log(apiPath)
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