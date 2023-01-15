



function switchPicture(isNext){
	console.log("switchPicture", window.opened_img_desc)
	if (window.opened_img_desc)
	{
		var next_img = isNext ? window.opened_img_desc.next_data : window.opened_img_desc.prev_data;
		console.log("nextImage:", next_img)
		if (next_img)
		{
			var $next_img = create_viewer_img(next_img);
			var $prev_img = $("#viewer-img-container > img");
			$next_img.addClass( isNext ? 'translate-off-right' : 'translate-off-left' );
			$("#viewer-img-container").append($next_img);
			$next_img.offset();

			$prev_img.one("webkitTransitionEnd otransitionend oTransitionEnd msTransitionEnd transitionend", function(e) {
				$(this).remove();
			});
			$next_img.removeClass( isNext ? 'translate-off-right' : 'translate-off-left' );
			$prev_img.addClass( isNext ? 'translate-off-left' : 'translate-off-right' );
			window.opened_img_desc=next_img;
			setVisibility();
		}
	}
}

$('#viewer-prev').click( function(){ return switchPicture(false) });
$('#viewer-next').click( function(){return  switchPicture(true)  });

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
		$('#viewer-prev').click();
	}
	else if (e.which == 39) {
		$('#viewer-next').click();
	}
});

function setVisibility()
{
	$('#viewer-next').css('visibility', !window.opened_img_desc.next_data ? 'hidden': 'visible')
	$('#viewer-prev').css('visibility', !window.opened_img_desc.prev_data ? 'hidden': 'visible')
}



function create_viewer_img(img_desc)
{
	img_desc.next_data = img_desc.next();
	img_desc.prev_data = img_desc.prev();
	var $window = $(window);
	var trans_x = ($window.width()/2  - img_desc['translation-origin'].left ) + 'px';
	var trans_y = ($window.height()/2 - img_desc['translation-origin'].top  ) + 'px';

    var $img = $('<img>')
	$img.attr('sizes', "(max-width: 512px) 512px, (max-width: 1024px) 1024px, 3000px")
	$img.attr('srcset', img_desc['src-set'])
	$img.attr('src', img_desc['src'])
	var transform = 'translate( calc( -50% - ' + trans_x +'), calc( -50% - ' + trans_y + ' ) ) scale(0)';
	$img.attr('data-close-transform',transform)
	
	return $img;
}

function open_viewer(img_desc)
{
	window.opened_img_desc = img_desc;
	var $img = create_viewer_img(img_desc)	
	$img.css({'transform' : $img.attr('data-close-transform') });
	$("#viewer-img-container > img").replaceWith($img);
	setVisibility();
	
	//reflow
	$img.offset();
	$('body').addClass("viewer-open");
	$img.removeAttr('style')
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
		$('#viewer-next').css('visibility', 'hidden')
		$('#viewer-prev').css('visibility', 'hidden')
		$('body').removeClass("viewer-open");
		$('body').removeClass("viewer-closing");
	});
	window.opened_img_desc = null;
}

$('#viewer-back').click( function() {
	close_viewer()
	return true;
});



function open_grid_element($a)
{
	var f = function($a)
	{
		if ($a.length == 0)
		{
			return null;
		}
	
		var offset  = $a.offset();
		offset.top  = (offset.top - $(window).scrollTop()) + $a.height()/2;
		offset.left = (offset.left - $(window).scrollLeft()) + $a.width()/2;
		return {
			'translation-origin': offset,
			'src-elem-id': $a.attr('id'),
			'src-set': $a.attr('data-srcset'),
			'src':  $a.attr('data-src'),
			'name': $a.children('span').text(),
			'next': function() { return f($a.next('a')); },
			'prev': function() { return f($a.prev('a')); }
		}
	}
	open_viewer(f($a));
}

function open_print_element($div)
{
	if ($div.length == 0)
	{
		return null;
	}
	
	var offset  = $div.offset();
	offset.top  = (offset.top - $(window).scrollTop()) + $div.height()/2;
	offset.left = (offset.left - $(window).scrollLeft()) + $div.width()/2;
	open_viewer({
		'translation-origin': offset,
		'src-elem-id': $div.attr('id'),
		'src-set': $div.children("img").attr('srcset'),
		'src':  $div.children("img") .attr('src'),
		'name': $('#title').text(),
		'next': function() { return null },
		'prev': function() { return null }
	});
}

$(document).ready(() => {

	$('.gallery > a').click( function(event){
		var $a = $(event.delegateTarget)
		open_grid_element($a);
	});
	
	$('#image').click( function(event){
		var $div = $(event.delegateTarget)
		open_print_element($div);
	});
});