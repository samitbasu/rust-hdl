# Cybersecurity

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
