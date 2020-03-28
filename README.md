# Lazing
 
A macro like lazy_static can initialize static variables.

# Usage

```rust
use std::ops::Deref;
#[lazy]
static NAME: String = "Hello".to_owned();

fn main() {
    println!("{}",NAME.deref());
}
 
```