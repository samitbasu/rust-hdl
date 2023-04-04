# Python Based Approaches

## MyHDL

MyHDL is an attempt to do something entirely different.  MyHDL uses Python as the
language of choice, and uses an asynchronous function model for capturing expected
behavior.  The Python library then translates those functions into Verilog that you
feed to your toolchain.  MyHDL is meant to tackle not just FPGAs, but ASICs and as
such, can be a bit limiting for FPGA use, since FPGAs have features that ASICs do 
not (generally) have, and those features can make a big difference in the difficulty
of describing your design.

## MiGen, FHDL

Unfortunately, I have no direct experience with either of these.  They live firmly
in the Python ecosystem, and they have built some pretty impressive libraries of
functionality.  While I spent some time writing firmware in MyHDL, I left the Python
ecosystem and started working on RustHDL before I fully explored all the possibilities.

I did note a couple of challenges with my Python based firmware:

### Python code cannot be known to be correct or incorrect unless it is tested. 
That means that no matter how simple the function, you don't know if it is even 
syntactically valid until you have exercised it in some way.  I'm sure that's a good thing, but I
found it annoying.  It was difficult enough to focus on the correctness of the design
and the logic without worrying that somewhere I had misspelled a signal name, and
so that branch would fail when invoked.  One can argue that all code must be 
tested to be correct, and that is certainly true.  But the contracts provided
by strongly typed languages like Rust mean you can focus your energy on the 
tests that look for edge cases and behavior problems.

### Generated code needs to be understandable.
There are still differences between CPUs and FPGAs we have not discussed, but it
is entirely possible/trivial to design an FPGA circuit with a few lines of code that
cannot actually be constructed.  If your designs are strict compositions of existing
pieces (SoC style), you may never encounter this problem.  And for many problems
connecting high level constructs together to accomplish an overall goal or pattern
is a reasonable solution.  However, if you are building your own components, with
customized behavior, then you will sooner or later, write something that cannot
be built.  For example, you will ask the FPGA to build a circuit that 
multiplies integers, dynamically indexes into a register, or performs some 
weird operation called "division" that the FPGA simply cannot do.  Or you may
cram-fooey too much into a single operation, forcing the entire FPGA to crawl along.
The food factory analogy is relevant here too (beginning to think I should have 
just written a book titled "The FPGA Food Factory Analogy").  In a normal factory,
you typically establish a "critical time" - which is how much time the longest 
indivisible operation in your process takes to complete.  Let's say this is 20 minutes
because the longest operation involves some intricate process.  That 20 minutes
sets the standard for every other step in the process.  For a simple, linear
type of workflow, you assume each step in the process takes no more than 20 minutes.

As a result, you can advance the state of the food pipeline every 20 minutes
like clock work.  This is precisely how FPGAs are designed and programmed.  Ok, 
not precisely, but close enough.  If you add a step to the process that takes 30 minutes,
then the whole process must slow down by 50%.  The same thing can happen when
programming an FPGA.  The tools will try their hardest to do what you ask, but you may end
up with a step in which all kinds of complexity are being called into order to
satisfy what you asked for.

Most of the time, this is not what you want, and not what you intended.  It's usually
a case of incorrect expression of the idea ("when I said make a salad, I meant just
some lettuce on the side"), or the idea itself needs to be broken down into smaller
simpler steps ("Let me set up a line to make salads and then inject them at this point").
In the later, you have "pipelined" the process.  Taken a single step ("make a salad"),
and broken it down into lots of smaller steps ("chop tomatoes").  These smaller steps
would then get their own stations in the food factory, and you could run at 20 minute
intervals again.

My point (yes, I had one), is that the information about where your instructions
are too complicated will come from the FPGA toolchain, and not from your high level
language.  A few lines in Python can easily generate a design that looks great and
simulates in Python just fine, but causes your toolchain to become indignant.  
When this happens, you must look at the Verilog code, since that is the contract
between you and the toolchain.  

This is where you will pay the price for the high level simplicity.  By abstracting
away the details, you could very well end up with `widget_0_1_2_3_4_5` violating
some hardware constraint, without any idea what that means.  This is a common problem
with translating to Verilog.  You often have to replicate, and "squash" hierarchies to
get it to behave the way you want/expect.  As a result, you end up with funkafied 
identifiers, monolothic mega-functions, global everythings, and all kinds of 
nastiness.  This is fine, as long as everything works as intended.  But if it doesn't,
you can be left with no clue as to what went wrong and why.

### Simulation speed matters

One of the important differences between hardware and software design is the importance
of test cases.  To test a hardware design, means you must either simulate it, or 
synthesize it (or both).  If you simulate a design, it means that you ignore some
of the lower level details of how the hardware actually works, and try to verify that
the behavior of a circuit matches your expectations.  This can take a really, really
long time.  If you are within a factor of 100 to 1000 of a real FPGA, you are doing well.
A fast mutli-GHz, multi-core CPU can struggle to model a massively parallel design 
running at a glacial 100MHz.  That 10-20X difference in speed can easily be 
overwhelmed by the fact that your CPU (when it's simulating an FPGA) can only perform
a handful of operations at a time, while that FPGA design may have hundreds or
thousands of operations running simultaneously.  On top of that, FPGA simulation is
a scary non-linear problem, and requires iterations.  It's hard to parallelize
(ironically), and a ton of work has gone into making FPGA simulation performant.

Can I illustrate my point?  Sure.  Why not.  Let's say we want to go back to the
temperature measurement problem.  This time, we want to measure the temperature 
over some long wires, using a digital protocol (like SPI or I2C).  Because the temperature
sensor is sitting far away, it's possible that occaisionally, the temperature
chip will not answer our request for a temperature reading.  In those cases, our
recovery procedure is to wait up to 1 second, and then try again.

Easy enough.  Let's assume our FPGA clock is running at 100 MHz, so that a 1 second
wait is simply the amount of time it takes to count to 100 million.  Hmm...
That can take a simulator a couple of minutes to do, at least for anything nontrivial.
I know!  I'll just shorten the count from 100 million to something smaller for
test purposes.  I'll use 100 thousand as the count instead of 100 million.  That's
quick to simulate, right?

Sure!  And when you changed the count how did you do it?  Did you change the
size of the counter?  Does it have enough bits for the actual applications?
Is the constant hard coded?  Or passed in?  Do you see the problem?

Modifying designs to make them faster to test is usually a bad idea.  It may be
unavoidable, but in general, testing the same code you plan to run on the hardware
is the best way of making sure it is actually correct.  So a simulation that runs
10X or 100X faster can make a difference in how much of your code you can realistically
test, and that matters!
