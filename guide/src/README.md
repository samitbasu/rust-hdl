# FPGA Programming with RustHDL

Let's start with the basic questions you may be asking yourself:

- What is RustHDL?  
- Why do we need it?  
- What is an FPGA anyway!  
- Where are my shoes?

These are all valid, and important questions, and I hope to answer at least a couple
of them in the course of this guide (did you check outside the front door?).  Let's start with
the most basic one.

## What is an FPGA anyway? (and who needs one?)

That is a great question!  And one I am not really planning to answer.  There are some great
resources available if you have never worked with an FPGA before.  The short version is that an 
FPGA can be thought of as collection of digital logic (and analog) circuits that are packaged together
and that can be reconfigured through software.  This is important to understand, since there is a 
fundamental difference between FPGAs and the CPUs that have become ubiquitous and dominant in the
electronics industry.  FPGAs are fundamentally *massively parallel devices*.  As such, they are 
not just really fast processors.  In fact, in terms of raw clock speed, modern CPUs and
(even some microcontrollers) will run circles around an FPGA.  CPUs are available in ever increasing
speeds and with ever more cores and capabilities.  Need a multi-core microcontroller that runs
at 500Mhz?  No problem.  CPUs clocked at 4GHz?  Tons of high speed DRAM?  Yup.  System-on-a-chip
that contains a bunch of nifty peripherals?  Yeah - you can use those too.

So why am I starting out with a list of reasons why you don't need an FPGA?  Primarily because FPGAs 
are really good at certain tasks, but to use them effectively is quite difficult.  RustHDL is my effort 
to make them less difficult to use, but it doesn't make the underlying complexity disappear.  You may say
"So what do FPGAs do well, then?".  Great question!  Glad I asked it.  There are a few areas
where FPGAs still hold an advantage over other digital solutions.

- Deterministic systems!
- High speed I/O
- Low power designs
- Cybersecurity

Let's take these in turn.

### Deterministic systems

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

### The Kitchen Analogy

CPUs are fast, but they are sequential devices.  Here's another way to think
about it.  A CPU is sort of like a person working in a kitchen.  A multi-core CPU
is sort of like multiple people working in the same kitchen.  When working together
you can create some amazing dishes/meals, with each of you focusing on some specialty
and moving around the kitchen in such a way as to share resources.  If you both need
the same knife or pot, you need to wait for someone to finish.  If you are sharing
the preparation of a dish, you need to hand it off at the right moment.  You may have
many tasks running in parallel (oven, mixer, blender, etc), but you can only focus
on one at a time.  And the resources are typically shared between the various people
in the kitchen.

An FPGA, on the other hand, is more like a food factory.  Imagine you need to crank out
thousands of identical (or very similar) dishes, with a high degree of predictability
(determinism).  The most efficient way to do that is to set up a large parallel 
assembly-line like operation, in which each station does only one task, and throughput
is achieved by increasing the number of stations doing that task.  That means
that each station has a dedicated set of resources (e.g, a cutting board, knife, onions),
but that you may have many of those stations operating in parallel to increase
the number of onions chopped per hour.

The second part of that food factory analogy is the interconnect.  If you have 
ever seen the inside of a food factory, it has equipment meant to connect the different
stations together.  Conveyor belts, usually, but it could be vehicles, robots, 
etc.  These move partial products from one part of the factory to another.  

Now, the reason I like the factory/kitchen analogy is that it fits on a bunch
of different levels.  A factory is good for producing at scale, identical (and often simplified)
recipies.  A kitchen excels at flexibility, improvisation, and adaptation.  Did you 
plan on having a block of Tofu, only to find someone else already used it?  Find a 
substitute!  Or stop and go get some.  Or make it.  Or bother a neighbor.  You get the
idea.

Factories, on the other hand, do not deal well with shortages.  How do you adapt an
onion station when there are no onions?  

This tradeoff between flexibility, and adaptability and the ability to scale is key
to why FPGAs are still relevant.  They have fewer applications these days than they 
used to, but there are still plenty of applications where an FPGA can elegantly and
efficiently solve a problem that is very hard to do on a microprocessor.  Incidentally,
you can, of course, build microprocessors on FPGAs, and some FPGAs come with CPUs
built in.  I won't focus on either of those at this point.  A microprocessor on an 
FPGA is an excellent way to bridge the gap between the two technologies, but requires
some fairly advanced techniques that we won't start with.  

## High Speed I/O

Another area where FPGAs excel is at high speed predictable I/O.  Suppose you are
building yourself a radio, and want to sample some intermediate signal at a couple 
of MSPS (Million samples per second).  It happens.  That means that some dedicated
circuitry is going to be sending you data at some pretty high rate.  How high?  
Well, assume the samples are 16 bits each, and you are shooting for 2 MSPS.  That 
means you need about 32 Mbits/second to keep up with the incoming data.  Sounds easy, 
right?  I mean, we have ubiquitous 1 Gbit/second ethernet everywhere.  What's a 
32 Mbit signal stream?

But it is actually not so easy.  That chip (an analog digital converter chip) will send 
a constant stream of data at high speed.  You might be able to read that with a microcontroller,
but it won't be easy.  Even if the chip satisfies some standard protocol, like SPI,
it may not send data the way your hardware SPI controller expects.  The chip manufacturer
has probably assumed that this is Your Problem.

FPGAs can solve these types of problems through a combination of 3 pieces:

- They typically have dedicated circuitry that deals with high speed I/O, and most high
speed interfaces operate the same or similar way.  These circuits speak the various electrical
protocols used by standards such as LVDS, etc.
- They allow you to quickly pack high speed, narrow data streams, into lower speed, wide
data streams.  A standard technique is to take that 32 Mbit/second stream and turn it
into a 2 million words/second stream, where the words are 16 bits wide.
- They typically contain hardware to do the reverse too - to take wide, slow streams and
convert them back into fast, narrow ones.

## Cybersecurity

Who doesn't worry about cybersecurity these days?  CPU software ecosystems are extraordinarily
complex.  And it is becoming increasingly difficult to ensure a system cannot be tampered with.
The flexible nature of the CPU means it can also be repurposed/subverted so that it does
something you never intended.  While it's an oversimplification to say that FPGAs don't suffer
from the same problem, the "factory" nature of their construction certainly makes it more
difficult.  The spectre of undefined behavior can pop up in FPGA systems (particularly at the
boundaries to the real world), but it's nothing like you see in software systems.  FPGA designs
tend to be built out of smaller, simpler components, strung together in complex topologies.  They
are harder to compromise.  Or at least it feels that way.

Great!  I've either convinced you that an FPGA is a great solution to some problem
you are wrestling with, or bored you to tears.  Either way, let's look at the problem with
programming FPGAs.

# FPGA Programming History In A Nutshell

I'm no expert on the history of FPGA programming.  However, I can point out some
guideposts in the area of FPGA programming, and my experience with them to date.
Your mileage may, of course, vary.  

## Verilog

Verilog was originally a language for simulating the behavior of digital circuits
that over time, became repurposed for FPGA programming.  FPGA programming is generally
"structural", and not "behavioral".  Going back to the kitchen analogy (again?? Is this
guy permanently hungry?), an FPGA program is like a floor plan for a factory.  It tells
you what sections are set up to do what function, and tells you how they are connected
to each other.  It does not (necessarily) tell you how that system behaves.  This is
exactly unlike CPU programming, which is a series of directed instructions, like a recipe.
In most software languages, you provide a series of instructions, or maybe you describe
the desired outcome.  In both cases, the CPU ends up with a stream of instructions telling
it to chop this, dice that, and then add the rice.  

Verilog is the lingua franca of the FPGA world.  And it is still the main way FPGAs are
programmed.  All of the newer technologies/languages ultimately generate Verilog or something
equivalent to it.  The toolchains, after all, expect Verilog as input.

So are we done?  Maybe!  For many FPGA developers (particularly those with hardware backgrounds),
Verilog or VHDL are the end of the story.  Much the same way for many CPU programmers, assembly
language or machine code are sufficient.  End of story.  For the rest of us, the goal is
to make developing FPGA programs slightly less traumatizing.  Compiler technology has come
a long, long way in the past several decades.  The Rust compiler (see!  I hadn't forgot
the purpose of this book) is incredibly sophisticated, and it goes through all kinds of hoops
to make sure you don't do silly things.  

Verilog lacks those same safeties.  It gives you, as the developer, ultimate power and
flexibility, with no guide-rails.  There are carefully constructed lists of rules regarding
how to use Verilog in ways that give up much of its flexibility in favor of readability.

## Lucid

Lucid is an excellent example of how Verilog can be improved.  Developed by 
what is now Alchitry Labs, Lucid is a higher level language that can be translated
into Verilog by the Alchitry IDE.  Lucid introduces things like types, structs, 
and other concepts that we will encounter.  These are standard
programming concepts that we think about all the time, but which are typically
absent in standard Verilog code.  At least as of now (Nov 2021), Lucid is tied to the
Alchitry IDE, and while I enjoy using Alchitry products, I also need to use FPGAs from
other vendors and suppliers.

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
That means that no matter how simple the function, you don't know if it is even syntactically
valid until you have exercised it in some way.  I'm sure that's a good thing, but I
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

## Goals

Modern projects have a list of goals and anti-goals that they list.  These
goals s


RustHDL is a technology I developed after trying a number of other methods for programming FPGAs.  It attempts to bring some of the great qualities of the Rust programming language to bear on the problem of programming
FPGAs.  In particular, it focuses on 