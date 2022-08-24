# apipe
 A simple annonymous UNIX pipe type.

 ## Usage

 ### try_from(&str)

 The probably easiest way to create a pipe is by parsing a command string:

 ```rust
 use apipe::CommandPipe;

 let mut pipe = CommandPipe::try_from(r#"echo "This is a test." | grep -Eo \w\w\sa[^.]*"#)?;

 let output = pipe.spawn_with_output()?
                  .stdout();
     
 assert_eq!(&String::from_utf8_lossy(output), "is a test\n");

 ```

 ### Pipe Command Objects

 Another way is to create the individual Commands and then contruct a pipe from them:

 ```rust
 use apipe::Command;

 let mut pipe = Command::parse_str(r#"echo "This is a test.""#)? | Command::parse_str(r#"grep -Eo \w\w\sa[^.]*"#)?;
                  
 let output = pipe.spawn_with_output()?.stdout();
     
 assert_eq!(&String::from_utf8_lossy(output), "is a test\n");

 ```

 `Command`s can also be constructed manually if you want:

 ```rust
 let mut command = Command::new("ls").arg("-la");
 ```

 ### Builder

 Finally, there is a conventional builder syntax:

 ```rust
 use apipe::CommandPipe;

 let mut pipe = apipe::CommandPipe::new();

 pipe.add_command("echo")
     .arg("This is a test.")
     .add_command("grep")
     .arg("-Eo")
     .arg(r"\w\w\sa[^.]*")
     .spawn()?;
     
 let output = pipe.output()?
                  .stdout();
     
 assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
 ```
