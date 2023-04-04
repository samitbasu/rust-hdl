---
sidebar_position: 98
---

# Roadmap

It's difficult to generate an accurate roadmap for a project like RustHDL, given that it's not sponsored and so vast in scope.
Nonetheless, here is the current set of milestones/priorities:

- Documentation!  The surface area of RustHDL is large, and it needs tons of documentation.  This is the main goal for the forseable future.
    * The crates need to be reorganized.  I originally had a series of smaller crates, but that gets to be a pain during development.  Now that things
      have mostly stabilized, smaller crates lead to faster compile times, and a better feature-controlled user experience.
    * Macros need to be documented, like `dff_setup`.
- Tutorials on the BSPs and using Hardware.  While its fun to play with simulations, the real hardware is generally more ~~frustrating~~ satisfying.
I have the following boards on hand (or used to), so I can do a basic tutorial on each of these as I go.
    * Alchitry Cu - this is an inexpensive ICE40-based board that can be programmed using only open source tools.  Probably one of the least expensive
    ways to get started.  I have a few of these, and I'd like to use them as the primary platform for the simple examples.
    * Alchitry Au - this is far more powerful, and includes the DDR and an Artix-7 FPGA.  You need to use commercial toolchains for this, but it's
    still reasonably priced for an FPGA board.
    * Orange Crab - this is a fun board based on the Lattice ECP5, and includes a LPDDR and a super-small form factor.  It can also be targetted with
    open source tools.
    * [OpalKelly](https://www.opalkelly.com) XEM6010 - an oldie, but great FPGA SOM based on the workhorse Spartan-6 from Xilinx.  The toolchain is way deprecated.
    I think this thing is great, but you can't get it anymore.  So...
    * [OpalKelly](https://www.opalkelly.com) XEM7010 - the modern version of the XEM6010.  Includes an Artix-7 and DDR.  RustHDL wraps the `FrontPanel` API to
    make working with OpalKelly modules easier to use.
    * Lattice ECP5 development board
    * More...?
- Analysis
    * I have the start of a timing analysis module in process in RustHDL.  I would like to finish it.  It constructs a full dataflow analysis of your
    design, and then looks for unprotected time-domain crossings.
- Make a `1.0.0` release.  This will be an official release, and then breaking changes will become more painful.  So it needs to be done
when there is a significant stability in the design.  I do not intend to make any substantial changes before releasing `1.0.0`.  Just documentation
and tweaks to improve usability.
- Future releases will focus on a richer set of widgets and more analytics.
- DSP blocks
    * FFT - at the least a basic decimation-in-time FFT engine written in RustHDL
    * CORDIC - arctangent's in hardware, baby!
    * FIR filters - I have one now, but it doesn't really synthesize properly on Lattice FPGAs
- Nanoprocessors
    * A number of applications would benefit from the hardware-reuse of a processor, but do not need/want
    the overhead of a full blown uC (like a RISC-V core).  For those, it would be nice to be able to
    define state machines with reusable ALUs and other micro-processor-like features.
- Communication Bridges
    * FPGAs with buttons and LEDs are definitely cool.  But even better are FPGAs that can serve as low-drama, high speed compute
    engines.  To do that, you need to get data in and out of them at speed.  USB is one choice.  But what about Ethernet, PCI-e, etc?
- Backend
    * I have run into a depressing number of bugs in commercial toolchains.  So I would like to start to move synthesis into
    RustHDL itself.  Yeah - I'm nuts.
- Place-and-route
    * [NextPNR](https://github.com/YosysHQ/nextpnr) is awesome.  I ran into problems trying to use it in one of my larger designs.  I'm not sure what to do about that.
    
