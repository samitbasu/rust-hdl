# Verilog

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
