#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {

    println!("\x1b[35;103;6;1m[WithColor]: Hello, Arceos!\x1b[0m");
}
