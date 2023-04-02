# Saving those Joules

Can you doe a lot with those new 400MHz Cortex-M4 ARM microprocessors?  Sure!  Are there any downsides?
Of course not!  

Well...  OK.  A few.  Here are some things to think about

- Check out how much power those processors use when doing nothing.
- Double check that number.
- Have a lie down.

With an FPGA, in general, you pay only for what you use.  You can usually clock it really slowly until
something interesting happens, at which point, you wake up and do stuff.  Or, you can just use parallelism and trade surface area for speed.  Generally, power usage goes up with clock speed.  Check
out this [paper](https://www.mecs-press.org/ijieeb/ijieeb-v4-n5/IJIEEB-V4-N5-7.pdf).  Cutting the clock speed in half means half the dynamic power usage (generally).

A good example of this tradeoff is trying to do trig functions quickly.  If you want to do e.g., 1 million cosines/sec for some reason, you probably need a hefty microcontroller.  Something that is
running at least 100 MHz or so, since a cosine is unlikely to take a single clock cycle through its
APU and you have other things you need to do too.  An FPGA, on the other hand, can compute a cosine
in a single clock cycle (regardless of the speed) if it's sufficiently pipelined, and you have enough
area.  So that sets the speed at 1 MHz.  That could be 1% of the power needed to keep the microcontroller doing the same work.  Food for thought.