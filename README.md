# Dandy

Dandy is a Rust library for some operations on DFAs, NFAs with an
accompanied CLI and website. It incorporates a format for denoting
DFAs and NFAs as well. This repo is very much work in progress.

The [library](dandy) is published to [crates.io](https://crates.io/crates/dandy)
and contains a more detailed readme.

## How to use

The CLI isn't done yet, but the `dandy-cli` folder is what is going
to become the binary. Change the main function and do `cargo run` to
run it all.

You can also build a (very simple) website which serves to check if
two DFAs denoted in the format is equivalent or not. Building the
website is done with `cd dandy-wasm` and `./build.sh`, and the output
website is located in `web-build`.

## Features

* Parsing DFAs/NFAs from the specified format
* Evaluating the DFA/NFA under some string
* Converting between DFAs and NFAs (DFA to NFA uses powerset construction with inaccessible states removed)
* Checking equivalence between two DFAs or NFAs
* Printing DFAs/NFAs as tables that can then be parsed again
* Some wasm bindings and a simple website
