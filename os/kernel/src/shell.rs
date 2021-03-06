use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use std::str;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) {
    loop {
        kprint!("{}", prefix);
        let mut storage = [0u8; 512];
        let mut input = StackVec::new(&mut storage);
        loop {
            let byte = CONSOLE.lock().read_byte();
            kprint!("{}", byte as char); //vs &byte?

            if byte == 0x00 {
                // ignore these bytes, I don't know where they come from
            } 
            else if byte == 0x08 || byte == 0x7f {
                if input.len() != 0 {
                    kprint!("{}", 0x08 as char);
                    kprint!(" ");
                    kprint!("{}", 0x08 as char);
                    input.pop();
                } 
            }
            // if this byte is the end of the input
            else if byte == b'\n' || byte == b'\r' {
                kprintln!("");
                let mut arguments: [&str; 64] = [""; 64]; // need to be inside this scope
                match Command::parse(str::from_utf8(&input).unwrap(), &mut arguments) {
                    Ok(command) => { 
                        match command.path() {
                            "echo" => { 
                                for i in 1..command.args.len() {
                                    kprint!("{} ", command.args[i]);
                                }
                                kprintln!("");
                            }
                            _ => { kprintln!("Error: unknown command: {}", command.path()); }
                        }
                    },
                    Err(Error::Empty) => { () },
                    Err(Error::TooManyArgs) => { kprintln!("\nError: too many arguments", ) }
                }
                break;
            } else {
                let result = input.push(byte);
                match result {
                    Ok(_) => { () },
                    Err(_) => { 
                        kprintln!("\nError: input is over 512 bytes long"); 
                        break;
                    }
                }
            }
        }
    }
}
