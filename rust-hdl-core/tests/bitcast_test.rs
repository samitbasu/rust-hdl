/*
In reference to issue: https://github.com/samitbasu/rust-hdl/issues/22

 Ok - let's see what's going on here. Here is the first line:
 self.character_ram.read_address.next = bit_cast(
   bit_cast(self.screen_x.val() >> 4) + bit_cast(self.screen_y.val() >> 4) * 40.into(),
  );
  The first one is basically addr = x / 16 + y/16 * 40. The generated code is

  character_ram$read_address = screen_x >> 32'h4 + (screen_y >> 32'h4 * 32'h28);
  which looks similar... However, there is a difference between the Verilog and the Rust context - the bitcast operator will mask off higher order bits. And Verilog will not. So my suspicion is that the difference comes down to the upper bits of screen_y and screen_x. In particular, let's consider a more basic case where we have a constant like:

      let x: Bits<8> = 0xFF.into();
      let y: Bits<6> = bit_cast(x);
  In RustHDL, this generates a value of 0x3F. But in verilog, it does not. That is because in Verilog,
  the bit_cast operation is a no-op. Yuck.

  To expose this, we need a case in which the lack of bit truncation influences a future
  value.  The simplest way I can think of is to simply bit cast twice, without
  explicity assigning to a narrow value in between.

*/

// First we need to build a test case that fails.
use anyhow::anyhow;
use rust_hdl_core::prelude::*;

#[derive(LogicBlock, Default)]
struct Issue11TestCase {
    pub f: Signal<Out, Bits<24>>,
    pub g: Signal<In, Bits<8>>,
}

impl Logic for Issue11TestCase {
    #[hdl_gen]
    fn update(&mut self) {
        self.f.next = bit_cast::<24, 32>(
            bit_cast::<32, 8>(self.g.val()) + bit_cast::<16, 8>(self.g.val()) * bits::<16>(2),
        );
    }
}

#[cfg(test)]
fn get_icarus_verilog_output(tb: &str) -> anyhow::Result<String> {
    std::fs::write("test_tb.v", tb).unwrap();
    let output = std::process::Command::new("iverilog")
        .args(["-tvvp", "-o", "test_tb.vvp", "test_tb.v"])
        .output()?;
    println!("iverilog output: {output:?}");
    let output = std::process::Command::new("vvp")
        .arg("test_tb.vvp")
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into())
}

#[test]
fn test_issue_11() -> anyhow::Result<()> {
    let mut uut = Issue11TestCase::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    let verilog_tb = r#"
module test;

  initial begin
    g = 8'hff; 
  end

  reg [7:0] g;
  wire [23:0] f;
  
  top uut(.f(f), .g(g));
   
  initial
     $monitor("%h", f);
endmodule //test
"#;
    let tb = format!("{verilog_tb} {vlog}");
    let sim = get_icarus_verilog_output(&tb)?;
    println!("Sim {sim}");
    println!("tb {tb}");
    let sim_value = u64::from_str_radix(sim.trim(), 16)?;
    // Build a simulator for RustHDL.
    let mut sim = Simulation::<Issue11TestCase>::new();
    sim.add_testbench(move |mut ep| {
        let mut x = ep.init()?;
        x.g.next = 0xFF.into();
        let x = ep.wait(1, x)?;
        sim_assert_eq!(ep, x.f.val(), sim_value, x);
        ep.done(x)?;
        Ok(())
    });
    sim.run(Box::new(uut), 10)
        .map_err(|err| anyhow!("{err:?}"))?;
    Ok(())
}
