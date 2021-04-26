const sum = as => as.reduce( (a,b) => a+b ,0 );
Array.min = (array) => Math.min.apply(Math,array);

var initialLayoutAchived = false;

function gallery() {

	const calculateLayout =  function(index,elem) {
		var   avgScale = {sum:0, len: 0}
		const width    = $(this).innerWidth();
		const children = $(this).children().toArray();
		const excess   = $(children[0]).outerWidth(true)-$(children[0]).innerWidth();
		$('children').each( function() { $(this).css( {'width':'','height':''});});


		var row = function(elems,lastRow){
			const heights = $.map(elems ,(e) => $(e).attr('data-height') );
			const min = Array.min(heights);
			var widths =  $.map(elems, (e) => $(e).attr('data-width') * min/ $(e).attr('data-height'));
			const scale = lastRow ? (avgScale.sum / avgScale.len) : 1.0002*sum(widths)/(width-elems.length*excess);
			widths = $.map(widths,function(e){return e/scale});
			const height = min/scale;


			if (scale < 1.0) {
				return false;
			}
			else {
				$(elems).each( function(i){
					$(this).width(widths[i]);
					$(this).height(height);
				})
				avgScale.sum += scale
				avgScale.len += 1;
				return true;
			}
		}
		
		var start = 0;
		var end   = 1;
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
		return $(this).innerWidth() == width;
	};

	{
		//Recalculate layout until it is stable
		let success  = true;
		let retries = 0;
		do
		{
			success = true;
			$('.gallery').each( function(i,e) {
				console.log('Calculation layout '+ i)
				success = calculateLayout.call(this,i,e) && success; //To avoid and elision...
				console.log("Success: " + success);
			});
		}
		while (retries++ < 2 && !success );
		if (success)
		{
			console.log("Achieved layout in " + retries +" attempts");
			if (!initialLayoutAchived){
				loadImagesWhenRdy();
				initialLayoutAchived=true;
			}
		}
		else
		{
			console.log("Failed to achive layout");
		}
	}
	
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

var originalLocation;

function loadImagesWhenRdy(){
	console.log('Arming image loading...')
	$(document).ready(function(){
		loadImages();
		location.hash = originalLocation;
	})
}

$(document).ready(() => {
	originalLocation = location.hash; 
	location.hash = "#"
	gallery()
});


