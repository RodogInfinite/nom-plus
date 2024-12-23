# Nom-Plus

`nom-plus` is a crate that extends the [`nom`](https://github.com/rust-bakery/nom) parser combinator crate, particularly that of error outputs. For full functionality, it must be used with [`nom-plus-macros`](https://github.com/RodogInfinite/nom-plus-macros). This crate includes some extra parsers, mostly for internal usage in the `nom-plus-macros` crate to generate error outputs, but that can be used independently.

--- 

# Objectives

1. Generate context-rich error messages that include:
  - The parser-chain that led to the error
  - The contextual information of that chain such as:
    - The functions they were called in
    - The signature of each function 
    - The location of each function 
    - The parsers of interest
      - The line numbers of each parser link in the chain
      - The input at each parser link in the chain
        - With configurable:
          - Truncation that has default length for readability
          - Level of inputs displayed up the chain from the error
      - The expected pattern for the parser where the error occured
2. Only compile the full ContextErrors in debug mode
  - Reduces performance overhead and memory footprint when running in release mode  
---

# Important Note:

This crate is **NOT** ready for use. Only the simplest `tag` example is currently working and there needs to be a major refactor to reduce the memory footprint of the ContextError Type. This refactor will halt any progress on supporting other Nom parsers for some time.

# Simplest Example:

```rust
use nom::bytes::complete::tag;
use nom_plus::prelude::*;
use nom_plus_macros::annotate_error;

#[annotate_error] // The attribute macro that generates the full ContextError in debug mode and only the minimal information in release mode
pub fn tag_error_example<'a>(input_name: &'a str) -> IResult<&'a str, &'a str, ContextError<'a>> {
    // `input_name` was chosen for demonstration purposes in the output below
    tag("world")(input_name) // also works with bindings and the `?` operator 
}

fn main() {
    // Expected to fail
    match tag_error_example("hello") {
        Ok((remaining, captured)) => {
           println!("Success! Remaining: {}, Captured: {}", remaining, captured);
        }
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
           eprintln!("{e:?}");
        }
        Err(nom::Err::Incomplete(_)) => {
           unimplemented!("Streaming is not supported at this time");
        }
    }
}

```
## Output:

```console
// Note: There is different coloration in the actual generated terminal output
// Note: Chaining is not yet implemented
error: ContextError
  --> crates/testing/src/main.rs // Location of the function where the error occurred
   |
 6 | pub fn tag_error_example<'a>(input_name: &'a str) -> IResult<&'a str, &'a str, ContextError <'a>> {
   |
 8 |     let x = tag("world")(input_name)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ error occurred here
   |
   = info: Found:
               input_name: "hello"

           Expected:
               pattern: "world"
```
