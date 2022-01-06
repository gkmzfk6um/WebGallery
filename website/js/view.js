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
		else if (src == "print")
		{
			url = window.location.pathname.replace(/^\/view\//,'/store/print/')
		}
	}
	return url;
};
function fetchMeta(link,success ) {
	if(success==null){
		success=function(){
			return history.replaceState(window.viewerMeta,"",window.location.href); 
		}
	}

	$.ajax({dataType: "json",url: link, success: function(data) {
		window.viewerMeta = data;
		success();
	}
	});
}
var setVisibility = t => {
	var visible = true;
	if (window.viewerMeta)
	{
		if (window.viewerMeta[t]==null)
		{
			visible = false;
		}
	}
	if (disableNav)
	{
		visible = false;
	}
	return $('#'+t).css('visibility', !visible ? 'hidden': 'visible');
}

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
	var vm = function(t) {return  window.viewerMeta[t];}
	var link = isNext ? vm('next') : vm('prev'); 
	fetchMeta(link,updatePage);
	return false;
}

$('#prev').click( function(){ return switchPicture(false) });
$('#next').click( function(){return  switchPicture(true)  });
$('#fullscreen').click( function() {
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
	
$('#back').click( function() {
	if( document.exitFullscreen ){
		document.exitFullscreen();
	}
	return true;
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


$(document).ready(() => {setBack(); setVisibility('next'); setVisibility('prev')});
