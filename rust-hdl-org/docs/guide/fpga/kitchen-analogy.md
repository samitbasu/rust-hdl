# True Parallelism (The Kitchen Analogy)

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

