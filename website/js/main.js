const sum = function(as){ return  as.reduce( function(a,b){return a+b; } ,0) }
Array.min = function(array){
	return Math.min.apply(Math,array)
}
var initialLayoutAchived = false;

function gallery() {
	$('.gallery').each( function(index,elem){

		var width = null;
		var i = null;
		//Loop to solve scrollbar appearing
		while(width != $(this).innerWidth() &&  i++ < 2){
			width= $(this).innerWidth();
			children=$(this).children().toArray();
			excess = $(children[0]).outerWidth(true)-$(children[0]).innerWidth();
			$('children').each(function() { $(this).css( {'width':'','height':''});});
			row = function(elems,lastRow){
				heights = $.map(elems ,function(e){return  $(e).attr('data-height'); });
				min = Array.min(heights);
				widths =  $.map(elems, function(e){ 
					return $(e).attr('data-width') * min/ $(e).attr('data-height')} );
				
				if(!lastRow){
					scale =  1.0002*sum(widths)/(width-elems.length*excess);
				}
				else{
					scale=1.0;
				}
				widths = $.map(widths,function(e){return e/scale});
				height = min/scale;


				if (scale < 1.0) {
					return false;
				}
				else {
					$(elems).each( function(i){
						$(this).width(widths[i]);
						$(this).height(height);
					})
					return true;
				}
			}
			
			start=0;
			end =1;
			while(end <= children.length+1){
				while(end <= children.length+1 && !row(children.slice(start,end),false)){
					end++;
				}
				if (end > children.length+1)
				{
					row(children.slice(start,end),true);
					break;
				}
				else{
					start=end;
					end++;
				}
			}
	}
		console.log('Image layout achived after ' + i + ' attempt(s).' );
		if (!initialLayoutAchived){
			loadImagesWhenRdy();
			initialLayoutAchived=true;
		}
	})
	
}

var timer;
$(window).resize(function() {
	clearTimeout(timer);
	timer = setTimeout(gallery,100)
});

function loadImages(){

	let loadImageNow = function(e)
	{
		let src= $(e).attr('data-src');
		let srcset= $(e).attr('data-srcset');

		let img = $('<img>')
		img.one('load',function() {
			$(this).css('visibility','visible');
			$(this).css('opacity', '1.0');
		})
		img.attr('srcset',srcset);
		img.attr('src',src);
		img.appendTo(e);
	};

	if (!('IntersectionObserver' in window) ||
    !('IntersectionObserverEntry' in window) ||
    !('intersectionRatio' in window.IntersectionObserverEntry.prototype)) {
		console.log('Loading all images (legacy)')
		$('.image').each( function() {
			loadImageNow(this);
		});
	}
	else {
		console.log('Loading intersection images');
		
		const config = {
			rootMargin: '300px',
			threshold: 0
		}

		let callback = function(entries,observer)
		{
			entries.forEach(function(entry)
			{
				if (entry.intersectionRatio > 0){
					observer.unobserve(entry.target);
					loadImageNow(entry.target);
				}
			});
		}
		let observer = new IntersectionObserver(callback,config);
		$('.image').each( function() {
			observer.observe(this);
		});

	}

}

function loadImagesWhenRdy(){
	console.log('Arming image loading...')
	$(document).ready(function(){
		loadImages();
	})
}

gallery();
