 <a href="foo">
 *bar*
 </a>
 <canvas id="DemoCanvas" width="500" height="200"></canvas>
 <script>
 var canvas = document.getElementById('DemoCanvas');
 //Always check for properties and methods, to make sure your code doesn't break in other browsers.
 if (canvas.getContext)
  {
   var context = canvas.getContext('2d');
   // Reset the current path
   context.beginPath();
   // Staring point (10,45)
    context.moveTo(10,45);
   // End point (180,47)
   context.lineTo(180,47);
   // Make the line visible
   context.stroke();
    }
 </script>

Here is some more random markdown

 - Stuff is cool
 - and junk
