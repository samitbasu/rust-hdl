# Deterministic systems

Lets suppose you have to do some incredibly boring and repetitive task,
like take the temperature of your Kombucha setup every so often.  Yawn.  
Piece of cake, right?  Fire up you PC, plug in a USB-based thermometer, write
some Python code, and go have lunch (and look for your shoes).  Hmmm,
but I want to do this constantly - like 24/7.  Oh.  Ok - so an embedded
solution.  Like a Raspberry Pi!  Simple enough (and often simple is the best solution).

Let's make it a bit harder.  Instead of reading the temperature every second, 
suppose we need to measure a _lot_ more frequently.  Like 10 times a second.
Or maybe 1000 times a second.  Ok - so regular operating systems don't really like
that kind of thing.  They will offer you the ability to run a task periodically, but
rarely guarantee that nothing will pre-empt your temperature measurement.  How about
a real time operating system (RTOS)?  Sure!  RTOSes can give you deterministic scheduling
and guarantee that your process will execute when you want.  But most have schedulers
that are limited to kilohertz ranges.  As we increase the speed with which we want
to measure our Kombucha, we find that RTOSes kind of shrug and wave us on.

Let's suppose you end up with an embedded solution that with enough wrangling
and coercion you can get to run at 10KHz - so that you sample the temperature
fast enough to make your brewmaster happy.  Chances are, you aren't doing a whole
lot more with your controller at this point.  What if I need a second one measured
at a different rate?  Um.  Ok - let's say the second one needs to measured at
20KHz.  Why 20KHz?  Well - because it immediately raises a problem.  On every
second sample, you will be trying to measure 2 temperatures simultaneously.  How
do you do that?  In general, a CPU cannot.  CPU cores typically manage a single
thread of execution at a time.  When you ask them to communicate with thermometer A
at time T, and then also ask them to communicate with thermometer B at time T,
the operating system will pick one of the two and then make the other one wait.
