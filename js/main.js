const sum = as => as.reduce( (a,b) => a+b,0)
function gallery() {

	$('.gallery').each( function(index,elem){
		width= $(this).innerWidth();
		children=$(this).children().toArray();
		pratio = window.devicePixelRatio;	
		//Calculate extra width
		//child = $(children[0]).children('img');
		//excess = $(children[0]).outerWidth(true)-child.outerWidth(true);
		excess = $(children[0]).outerWidth(true)-$(children[0]).innerWidth();
		
		$('children').each(function() { $(this).css( {'width':'','height':''});});
		row = function(elems,lastRow){
			heights = $.map(elems ,e=> $(e).attr('data-height')/pratio);
			min = Math.min(...heights);
			widths =  $.map(elems, e=> $(e).attr('data-width') * min/ $(e).attr('data-height') );
			
			if(!lastRow){
				scale =  1.0001*sum(widths)/(width-elems.length*excess);
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
loadImages();
