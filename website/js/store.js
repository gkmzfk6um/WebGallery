function resizeT(iterations)
{
    var bodyWidth= $(".body").width()
    $(".printImage").each( (i,e) => {
            console.log(e)
            const height = $(e).attr('data-height');
            const width = $(e).attr('data-width');
            $(e).height(height/width * $(e).width() )
        }
    )
    if ( $(".body").width() != bodyWidth )
    {
        if (iterations < 2)
        {
            resizeT(iterations+1)
        }
        else
        {
            console.log("Failed to layout store!")
        }
    }
}
function resize()
{
    resizeT(0);
}

function initStore()
{
    resize();
    var timer;
    $(window).resize(function() {
    	clearTimeout(timer);
    	timer = setTimeout(resize,100)
    });

    $(".printImage").click( (e) => {
        $(".printContainer").removeClass("floating")
        console.log($(e.delegateTarget).parents(".printContainer"))
        $(e.delegateTarget).parents(".printContainer").addClass("floating")
        resize()
    } )


    $(".printImage").each( (i,e) => {
	    const src= $(e).attr('data-src');
	    const srcset= $(e).attr('data-srcset');
	    let img = $('<img>')
	    img.one('load',function() {
	    	$(this).css('visibility','visible');
	    	$(this).css('opacity', '1.0');
	    })
	    img.attr('srcset',srcset);
	    img.attr('src',src);
        $(e).append(img)
    } )
}


$(document).ready(() => initStore());
