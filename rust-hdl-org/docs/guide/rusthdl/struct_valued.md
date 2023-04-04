# Struct valued signals

We have seen how Enums and Interfaces can help make your code more compact and readable.  There
is another abstraction you can use to simplify your code.  Interfaces allow you to group together
signals that are logically related into a named bundle (like a bus).  You can also group
together `bits` into a logically related bundle that can be treated as a single entity.  
While this is supported in RustHDL, it's not frequently that useful.  Nonetheless.

Suppose you have a set of signals to your circuit that all travel in the same direction,
but are different widths.  Any maybe some of the elements are enums.  Something like this

```rust
# use rust_hdl::prelude::*;
struct Foo {
   pub in_red: Signal<In, Bits<5>>,
   pub in_green: Signal<In, Bits<8>>,
   pub in_blue: Signal<In, Bits<8>>,
   pub in_alpha: Signal<In, Bits<6>>,
}
```

Instead, we can define a struct and annotate it with [LogicStruct], which makes it into a
type that can be used for a signal.
```rust
# use rust_hdl::prelude::*;
   #[derive(Default, PartialEq, LogicStruct, Copy, Clone, Debug)]
   struct Color {
       pub red: Bits<5>,
       pub green: Bits<8>,
       pub blue: Bits<8>,
       pub alpha: Bits<6>,
   }

   struct Foo {
       pub in_color: Signal<In, Color>,
       pub local_color: Signal<Local, Color>,
       pub out_color: Signal<Out, Color>,
   }

   impl Logic for Foo {
       #[hdl_gen]
       fn update(&mut self) {
           self.local_color.next = self.in_color.val(); // Copy the struct as a single atom
                                    // v-- Extract a single field using standard `.field` notation
           if self.local_color.val().alpha.get_bit(4) {
               self.local_color.next.red = 16.into(); // Assign to a single field of the struct
           }
           self.out_color.next = self.local_color.val();
       }
   }
```

From within the HDL kernel, you can access the fields of the struct as you normally would.  You can
also assign entire structs to one another, as well as individual fields of a struct.  The generated
Verilog is messy, and I don't use struct valued signals much.  But if you need to use them they are
there.
