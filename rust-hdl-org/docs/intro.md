---
sidebar_position: 1
---

# Tutorial Intro

Let's go from an empty setup to a functioning blinking LED on an actual FPGA!  Blinking an LED is a time-honored tradition of demonstrating that you can go from source to somehting that understands
the hardware it is running on.

## Mise en Place (Prerequisites)

Hardware projects can be a little more complicated than pure software ones.  So let's look at the prerequisites for this tutorial.

### The FPGA board

In principle, all you need is an FPGA board with an LED that you know how to program.  There are a number
of hardware specific bits and pieces that you need to get right for RustHDL to actually work on you device
of interest.  In particular, knowing what FPGA is being used is not sufficient in and of itself.  You 
need to know how it was configured and connected to make it do things.  In any case, for this tutorial,
let's start with the [Alchitry Cu](https://www.sparkfun.com/products/16526) board.  It's reasonably inexpensive, very well constructed and designed, and generally available.  But adapting this tutorial to other boards is quite straightforward.  Here is a pic of the Alchitry board ![Alchitry board](./img/16526-Alchitry_Cu_FPGA_Development_Board__Lattice_iCE40_HX_-03.jpg).

### The host OS

With the exception of [Yosys](https://github.com/YosysHQ/yosys), I do not know of FPGA tooling that runs on 
Mac OS X.  So I would recommending using either Linux or Windows for now.  This tutorial will assume Linux
(and comfort with the command line).  The exact Linux distribution doesn't matter per se, but the host OS
will need to provide tools that Rust does not.  And those will require the host OS package manager to be 
involved.  For now, I've picked an Ubuntu distribution.

### Rust basics

I would say that RustHDL requires `basic` understanding of how to code in Rust.  If you are comfortable with

- Strong types
- Algrbraic types (enums)
- Matching
- Basic expressions
- Value types

You should be good to go 👍.  You do not need to worry about lifetimes, references, or any of the more intermediate concepts.  I strongly recommend this _not_ be your first experience with Rust.  That should come
from the [book](https://doc.rust-lang.org/stable/book).  Once you have gotten an handle on that, this should
all seem quite simple.

I will assume, for example, you have already installed `cargo, rustup, rust, etc.` 

### Code Editor

** This is important! **  Let's face it.  Generally, IDEs for HDLs are pretty rough.  There aren't enough HDL users to really stimulate the ecosystem and get focus on good tooling.  This is ** not ** the case for RustHDL!  RustHDL uses Rust's syntax, and the analysis tools are capable of understanding most (if not all) of what RustHDL does under the hood.  As a result, you get great features like 

- code completion
- syntax highlighting
- warning and errors
- go to definition/declarations etc

even _inside_ your HDL code!  You can even use `vim` or `emacs` with these tools if you want to.  To me, 
the ability to piggy-back 🐖 on the broader community for things like IDE support are one of the big 
pluses of working in RustHDL instead of something more domain specific.  If you need help getting
set up, this [page](https://code.visualstudio.com/docs/languages/rust) from the vscode team is a great place to start.

### FPGA Toolchain

Beware the slings and arrows of outrageous FPGA toolchains!  There are many hurdles here, and progress is slow.  Suffice to say things are getting better, but you may still need to deal with a legacy toolchain.  The
Alchitry CU uses the iCE40HX FPGA from Lattice, and it is well documented by [Project IceStorm](http://www.clifford.at/icestorm/).  Installation with modern distros is simple.  For Ubuntu, it's

```bash
sudo apt install fpga-icestorm
```

Note that you need a place and route tool.  The one originally used (`arachne`) is no longer supported.  So I went ahead and also installed the `nextpnr-ice40` package.

```bash
sudo apt install nextpnr-ice40
```

## Ready Set Go

With everything in place, we can get started, and it will come together quickly.  First, we create a new
Rust project of the binary type.

```bash
samitbasu@samitbasu-virtual-machine:~/Devel$ cargo new blinky
     Created binary (application) `blinky` package
```

Next, we add the `rust-hdl` meta package as a dependency.  You will need the `fpga` feature

```bash
samitbasu@samitbasu-virtual-machine:~/Devel/blinky$ cargo add rust-hdl --features fpga
    Updating crates.io index
      Adding rust-hdl v0.44.2 to dependencies.
             Features:
             + fpga
samitbasu@samitbasu-virtual-machine:~/Devel/blinky$ 
```

We will also need the board support package for the Alchitry Cu board.  So lets add that too

```bash
samitbasu@samitbasu-virtual-machine:~/Devel/blinky$ cargo add rust-hdl-bsp-alchitry-cu
    Updating crates.io index
      Adding rust-hdl-bsp-alchitry-cu v0.44.2 to dependencies.
samitbasu@samitbasu-virtual-machine:~/Devel/blinky$ 
```

Next, we need to replace the contents of `main.rs` with the following

```rust
use rust_hdl::prelude::*;
use rust_hdl_bsp_alchitry_cu::pins::CLOCK_SPEED_100MHZ;
use rust_hdl_bsp_alchitry_cu::{pins, synth};
use std::time::Duration;

#[derive(LogicBlock)]
pub struct Blinky {
  pulser: Pulser,
  clock: Signal<In, Clock>,
  leds: Signal<Out, Bits<8>>,
}

impl Logic for Blinky {
  #[hdl_gen]
  fn update(&mut self) {
    self.pulser.enable.next = true;
    self.pulser.clock.next = self.clock.val();
    self.leds.next = 0x00.into();
    if self.pulser.pulse.val() {
      self.leds.next = 0xAA.into();
    }
  }
}

impl Default for Blinky {
  fn default() -> Self {
    let pulser = Pulser::new(CLOCK_SPEED_100MHZ.into(), 1.0, Duration::from_millis(250));
    Blinky {
      pulser,
      clock: pins::clock(),
      leds: pins::leds(),
    }
  }
}

```

```

## Getting Started









Let's discover **Docusaurus in less than 5 minutes**.

## Getting Started

Get started by **creating a new site**.

Or **try Docusaurus immediately** with **[docusaurus.new](https://docusaurus.new)**.

### What you'll need

- [Node.js](https://nodejs.org/en/download/) version 16.14 or above:
  - When installing Node.js, you are recommended to check all checkboxes related to dependencies.

## Generate a new site

Generate a new Docusaurus site using the **classic template**.

The classic template will automatically be added to your project after you run the command:

```bash
npm init docusaurus@latest my-website classic
```

You can type this command into Command Prompt, Powershell, Terminal, or any other integrated terminal of your code editor.

The command also installs all necessary dependencies you need to run Docusaurus.

## Start your site

Run the development server:

```bash
cd my-website
npm run start
```

The `cd` command changes the directory you're working with. In order to work with your newly created Docusaurus site, you'll need to navigate the terminal there.

The `npm run start` command builds your website locally and serves it through a development server, ready for you to view at http://localhost:3000/.

Open `docs/intro.md` (this page) and edit some lines: the site **reloads automatically** and displays your changes.
