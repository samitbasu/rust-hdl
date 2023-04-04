---
sidebar_position: 100
---
# References

RustHDL was inspired by and borrows from the following awesome projects!  They are listed in
no particular order...

- LucidHDL is a very cool HDL developed by the folks over at [Alchitry](https://www.alchitry.com).  I initially
started writing tooling to manipulate Lucid using Rust, and then realized that it would be better for
me to just use Rust itself.  But there is some great stuff on their website, and their book is excellent.
- AlchitryLabs is an IDE from [Alchitry](https://www.alchitry.com) that supports the LucidHDL.  They open sourced 
it and it is an impressive piece of software.
- For Python fans, [MyHDL](https://myhdl.org) is a Python based approach to generating HDL.  If you consider yourself a Python person, check it out.  After I left Verilog, I first rewrote a fair chunk of firmware using MyHDL and Python.  In the end, it wasn't for me, but many of the ideas are very cleanly expressed.
- _The_ open source toolchain that started it all is [IceStorm](https://clifford.at/icestorm) Open Source tool chain for the `iCE 40` FPGA, and an incredibly powerful concept.  In particular, Claire's work demonstrated a software-like path for handling FPGAs that was critical to a number of open source FPGA-centric projects. 
- [YoSys](https://github.com/YosysHQ/yosys) is the Verilog synthesis suite used by [RustHDL] to 
process generated Verilog and check a design for potential errors (static analysis).  
- [Icarus](http://iverilog.icarus.com/) is a verilog simulator that is software based, open source
and easy to use.  Before [RustHDL] got it's own simulator, I would write and simulate everything
using [Icarus].
- [Verilator](https://www.veripool.org/verilator/) I'd condsider this the professional option
for simulation (particularly of super complicated designs).  [RustHDL] doesn't come anywhere near
the simulation performance of [Verilator] yet, but the goal is to either generate Verilog that
[Verilator] can process, or adopt the same types of techniques described by the author in this 
[paper](https://veripool.org/papers/Verilator_Internals1_202010.pdf).
- [OpalKelly](https://opalkelly.com) - Excellent FPGA modules to use for pretty much any purpose.  Their
FrontPanel API is super easy to use, and RustHDL provides bindings to make it trivial.
