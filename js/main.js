const sum = as => as.reduce( (a,b) => a+b,0)
function gallery() {
	$('.gallery').each( function(index,elem){
		var width = null;
		var i = null;
		//Loop to solve scrollbar appearing
		while(width != $(this).innerWidth() &&  i++ < 2){
			width= $(this).innerWidth();
			children=$(this).children().toArray();
			//Calculate extra width
			//child = $(children[0]).children('img');
			//excess = $(children[0]).outerWidth(true)-child.outerWidth(true);
			excess = $(children[0]).outerWidth(true)-$(children[0]).innerWidth();
			
			$('children').each(function() { $(this).css( {'width':'','height':''});});
			row = function(elems,lastRow){
				heights = $.map(elems ,e=> $(e).attr('data-height'));
				min = Math.min(...heights);
				widths =  $.map(elems, e=> $(e).attr('data-width') * min/ $(e).attr('data-height') );
				
				if(!lastRow){
					scale =  1.0002*sum(widths)/(width-elems.length*excess);
				}
				else{
					scale=1.0;
				}
				widths = $.map(widths,e=>e/scale);
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
		if (1 == window.devicePixelRatio){	
			src= $(this).attr('data-src');
		}
		else{
			src= $(this).attr('data-src2');
		}
		var img = $('<img>')
		img.one('load',function() {
			$(this).css('visibility','visible');
			$(this).css('opacity', '1.0');
		})
		img.attr('src',src);
		img.appendTo(this);

	});
	console.log('Image loading started');
}
gallery();
$(document).ready(function(){
	loadImages();
})
