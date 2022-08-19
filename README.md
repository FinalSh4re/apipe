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
        .arg(r"\w\w\sa[^.]*");

    let output = pipe.spawn();

    assert_eq!(
        String::from_utf8_lossy(&output.unwrap().stdout),
        String::from("is a test\n")
    );
}
```
