# What is an FPGA anyway? (and who needs one?)

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

- [Deterministic systems](./deterministic-systems.md)!
- [True parallelism](./kitchen-analogy.md)
- [High speed I/O](./high-speed-io.md)
- [Low power designs](./low-power.md)
- [Cybersecurity](./cybersecurity.md)

Keep reading for more details!
