const urlParams = new URLSearchParams(window.location.search);
const disableNav = urlParams.has('stay')
const returnSrc  = () => {
	var name;
	if (window.viewerMeta)
	{
		name = window.viewerMeta['name'];

	}
	else
	{
		name = $("#picname").text();
	}
	var url = '/#' + name;
	if (urlParams.has("src") )
	{
		const src = urlParams.get('src');
		if (src == 'categories')
		{
			url = '/categories#' + name;
		}
		else if (src == "portraiture")
		{
			url = '/portraiture#' + name;
		}
		else if (src == "print")
		{
			url = window.location.pathname.replace(/^\/view\//,'/store/print/')
		}
	}
	return url;
};

//function fetchMeta(link,success ) {
//	if(success==null){
//		success=function(){
//			return history.replaceState(window.viewerMeta,"",window.location.href); 
//		}
//	}
//
//	$.ajax({dataType: "json",url: link, success: function(data) {
//		window.viewerMeta = data;
//		success();
//	}
//	});
//}
//var setVisibility = t => {
//	var visible = true;
//	if (window.viewerMeta)
//	{
//		if (window.viewerMeta[t]==null)
//		{
//			visible = false;
//		}
//	}
//	if (disableNav)
//	{
//		visible = false;
//	}
//	return $('#'+t).css('visibility', !visible ? 'hidden': 'visible');
//}

var setBack = () => {
	$('#back').attr('href',returnSrc());
}

function updatePage(isBack){
	if(isBack == null){
		isBack = false
	}
	document.title = window.viewerMeta['name'];
	$('#picname').text(document.title);
	if( !isBack){
		history.pushState(window.viewerMeta,"",window.viewerMeta['url']);
	}
	$('.viewer').children('img').css('opacity',0);
	var img = $('<img />').attr('srcset', window.viewerMeta['srcset'])
	img.attr('src', window.viewerMeta['path'])
	img.one('load',function(){
		$('.viewer').children('img').remove();
		$('.viewer').append(this);
	});
	setVisibility('next');
	setVisibility('prev');
	setBack();
}

function switchPicture(isNext){
	return false;
}

$('#prev').click( function(){ return switchPicture(false) });
$('#next').click( function(){return  switchPicture(true)  });
$('#viewer-fullscreen').click( function() {
	if (!document.fullscreenElement) {
		document.documentElement.requestFullscreen();
	}
	else {
		if (document.exitFullscreen) {
			document.exitFullscreen(); 
		}
	}
	return false;
});
	

if(!document.fullscreenEnabled) {
	$('#fullscreen').css('display','none');
}

$('body').keydown(function(e){
	if (e.which == 37) {
		$('#prev').click();
	}
	else if (e.which == 39) {
		$('#next').click();
	}
});

$(window).on('popstate', function(e) {
		window.viewerMeta = e.originalEvent.state;
		updatePage(true);
});



function open_viewer(img_desc)
{
	window.opened_img_desc = img_desc;
	var $window = $(window);
	var trans_x = ($window.width()/2  - img_desc['translation-origin'].left ) + 'px';
	var trans_y = ($window.height()/2 - img_desc['translation-origin'].top  ) + 'px';
	var $div = $('<div>');
	$div.css("position","absolute");
	$div.css("left",img_desc['translation-origin'].left);
	$div.css("top",img_desc['translation-origin'].top);
	$div.css("background-color","red");
	$("#viewer-img-container").append($div);

	console.log(img_desc);
    var $img = $('<img>')
	$img.attr('sizes', "(max-width: 512px) 512px, (max-width: 1024px) 1024px, 3000px")
	$img.attr('srcset', img_desc['src-set'])
	$img.attr('src', img_desc['src'])
	var transform = 'translate( calc( -50% - ' + trans_x +'), calc( -50% - ' + trans_y + ' ) ) scale(0)';
	$img.css({'transform' : transform });
	//$img.css({'transform' : 'translate( ' + trans_x +', ' + trans_y + '  ) scale(0)'});
	$("#viewer-img-container > img").replaceWith($img);
	$img.offset();

	$('body').addClass("viewer-open");
	$img.removeAttr('style')
	$img.attr('data-close-transform',transform)
}

function close_viewer()
{
	if (document.fullscreenElement) {
		if( document.exitFullscreen ){
			document.exitFullscreen();
		}
	}
	var $img = $("#viewer-img-container > img");
	$img.offset();
	$img.css('transform',$img.attr('data-close-transform'));
	$('body').addClass("viewer-closing");
	$('body').one("webkitTransitionEnd otransitionend oTransitionEnd msTransitionEnd transitionend", function(e) {
		$('body').removeClass("viewer-open");
		$('body').removeClass("viewer-closing");
	});

}

$('#viewer-back').click( function() {
	close_viewer()
	return true;
});


function open_grid_element($a)
{
		var offset  = $a.offset();
		offset.top  = (offset.top - $(window).scrollTop()) + $a.height()/2;
		offset.left = (offset.left - $(window).scrollLeft()) + $a.width()/2;
		var image_desc = {
			'translation-origin': offset,
			'src-elem-id': $a.attr('id'),
			'src-set': $a.attr('data-srcset'),
			'src':  $a.attr('data-src'),
			'name': $a.children('span').text(),
			'next': function() { open_grid_element($a.next('a')); },
			'prev': function() { open_grid_element($a.prev('a')); }
		}
		open_viewer(image_desc);

}

$(document).ready(() => {

	$('.gallery > a').click( function(event){
		var $a = $(event.delegateTarget)
		open_grid_element($a);
	}
	);
});