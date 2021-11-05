# os

A small os written in Rust. Based off the [tutorials by Philipp Oppermann](https://os.phil-opp.com/).

## Run

Install nightly cargo by doing `rustup toolchain install nightly` (Tested on `rustc 1.58.0-nightly (4961b107f 2021-11-04)`).
You need the [bootimage](https://github.com/rust-osdev/bootimage) crate installed as well: `cargo install bootimage`
Probably also some other stuff but It Works On My Machine™.
Make sure you have [qemu](https://www.qemu.org/) installed, and then you can do `cargo run`.
Hopefully it doesn't crash!
