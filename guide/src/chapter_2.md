# Chapter 2 - Signals

The `Signal` class represents a connection pathway (think conveyor belt in our
food factory analogy).  It carries a value from one location on the FPGA to
another.  Furthermore, the `Signal` has two attributes that are part of its
type signature:

- Direction: This gives you a hint as to whether the signal is meant to carry
  data away from this part, or if the signal is meant to bring data into this part.
- Kind: The second type argument for the signal tells you what kind of data the
  signal carries.

So in the case of `Signal<In, Clock>`, we know that the signal brings into the
circuit a clock line (which is a special type used to represent clock signals).
And in the case of `Signal<Out, Bits<8>>`, the signal carries 8 bits out from the
circuit.  That's basically it!  Not much magic there.  Let's look
