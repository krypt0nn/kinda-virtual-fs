<h1 align="center">🦀 kinda-virtual-fs</h1>

A Rust library to imitate virtual filesystem. Used by me to link files in my applications

# Example

```rust
use std::collections::HashMap;

use kinda_virtual_fs::*;

// File `../assets/icon.png` will be statically linked by the rust compiler
let storage = Storage::new(HashMap::from([
    ("icon".to_string(), Entry::new(include_bytes!("../assets/icon.png").to_vec()))
]));

let path = storage.map("icon").unwrap();

println!("Icon was saved as {}", path);
```

Author: [Nikita Podvirnyy](https://vk.com/technomindlp)

Licensed under [GNU GPL 3.0](LICENSE)
