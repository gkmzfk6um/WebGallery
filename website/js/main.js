const sum = function(as){ return  as.reduce( function(a,b){return a+b; } ,0) }
Array.min = function(array){
	return Math.min.apply(Math,array)
}

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
	})
	
}

var timer;
$(window).resize(function() {
	clearTimeout(timer);
	timer = setTimeout(gallery,100)
});

function loadImages(){
	$('.image').each( function() {
		src= $(this).attr('data-src');
		srcset= $(this).attr('data-srcset');
		
		var img = $('<img>')
		img.one('load',function() {
			$(this).css('visibility','visible');
			$(this).css('opacity', '1.0');
		})
		img.attr('srcset',srcset);
		img.attr('src',src);
		img.appendTo(this);

	});
	console.log('Image loading started');
}
gallery();
$(document).ready(function(){
	loadImages();
})
