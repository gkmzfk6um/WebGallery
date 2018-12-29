const sum = as => as.reduce( (a,b) => a+b,0)
function gallery() {

	$('.gallery').each( function(index,elem){
		width= $(this).innerWidth();
		children=$(this).children().toArray();
		$('children').each(function() { $(this).css( {'width':'','height':''});});
		row = function(elems){
			heights = $.map(elems ,function(e) {return $(e).attr('data-height')});
			min = Math.min(...heights);
			scales = $.map(heights, e => e/min );
			$(elems).each(function(i){
				newWidth = $(this).attr('data-width') / scales[i];
				$(this).width(newWidth); 
			});
			$(elems).height(min)
			outerwidths = $.map(elems, function(e) { return $(e).outerWidth(true);})
			innerwidths = $.map(elems, function(e) { return $(e).innerWidth();})
			sumowidth = sum(outerwidths);
			sumiwidth = sum(innerwidths);
			excess = sumowidth-sumiwidth;
			scale = sumiwidth/(width-excess)*1.0001 ;

			if (scale < 1.0) {
				return false;
			}
			else {
				$(elems).each( function(){
					newWidth = $(this).innerWidth() / scale;
					newHeight =$(this).innerHeight()  / scale;
					$(this).width(newWidth);
					$(this).height(newHeight);
				})
				console.log(elems.length)
				return true;
			}

		}
		
		start=0;
		end =1;
		while(end <= children.length){
			while(end <= children.length && !row(children.slice(start,end))){
				end++;
			}
			start=end;
			end++;
		}
	})
	
}

var timer;
$(window).resize(function() {
	clearTimeout(timer);
	timer = setTimeout(gallery,100)
});
gallery();
