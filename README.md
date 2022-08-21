# apipe
Anonymous UNIX pipe type in rust.

## Usage

```rust
use apipe::CommandPipe;

fn main() {

    let mut pipe = CommandPipe::new();

    pipe.add_command("echo")
        .arg("This is a test.")
        .add_command("grep")
        .arg("-Eo")
        .arg(r"\w\w\sa[^.]*")
        .spawn()
        .expect("Failed to spawn pipe.");

    let output = pipe.output();

    assert_eq!(output.unwrap(), "is a test\n");
}
```
