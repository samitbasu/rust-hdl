## Visitor pattern overhead

The use of the visitor pattern (while elegant) imposes a significant cost overhead
that I was surprised by.  When everything was in a single translation unit, the
performance was fine (2.6 seconds for the standard 100M clock benchmark).  However,
when things were split up into multiple translation units, the performance dropped
to 13+ seconds.  That is a significant loss.  Probably too much to really tolerate
at this stage.  It is a given that we will give away performance over time as we
stretch the core design to handle more scenarios and more sophisticated features.

However, I don't feel comfortable giving away the performance up front.  Using
the macro engine to generate the `update_all` and `has_changed` functions seems
like a small cost to pay to keep most of the performance gains in eliminating the
complex ARC/RC based design I had before.

## Performance and multi-threading

To achieve true multi-threading of the simulation, we must separate out
the state from the manipulation of the state.  For example

```rust
struct Widget {
    clock: Signal<In, Clock>,
    enable: Signal<In, Bit>,
    counter: DFF<Bits<6>>,
    strobe: Signal<Out, Bit>,
}
```

Now suppose that the logic does not mutate, but instead creates a
new structure for the signals.  Since the signals do not have wiring
internal to them anymore (they are just signals), we can do something
like this for `update`:

```rust
fn update(mut w: Widget) -> Widget {
    w.counter.clk.next = w.clock.val;
    if w.enable.val {
        w.counter.d.next = w.counter.q.val + 1;
    }
}
```
