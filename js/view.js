function fetchMeta(link,success=()=>{}) {
	$.ajax({dataType: "json",url: link, success: function(data) {
		window.viewerMeta = data;
		success();
	}
	});
}

function updatePage(){
	document.title = window.viewerMeta['name'];
	$('#picname').text(document.title);
	history.pushState({},"",window.viewerMeta['url']);
	$('.viewer').children('img').css('opacity',0);
	var img = $('<img />').attr('src', window.viewerMeta['path']).one('load',function(){
		$('.viewer').children('img').remove();
		$('.viewer').append(this);
	});
	var setVisibility = t => $('#'+t).css('visibility',window.viewerMeta[t]==null? 'hidden': 'visible');
	setVisibility('next');
	setVisibility('prev');
	$('#back').attr('href','/#' + window.viewerMeta['name']);
}

function switchPicture(isNext){
	var vm = t => window.viewerMeta[t];
	var link = isNext ? vm('next') : vm('prev'); 
	fetchMeta(link,updatePage);
	return false;
}

$('#prev').click( () => switchPicture(false) );
$('#next').click( () => switchPicture(true)  );
$('body').keydown(function(e){
	if (e.which == 37) {
		$('#prev').click();
	}
	else if (e.which == 39) {
		$('#next').click();
	}
});
