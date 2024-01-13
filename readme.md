# Dandy
Dandy is a Rust library for some operations on DFAs, NFAs with an
accompanied CLI and website. It incorporates a format for denoting
DFAs and NFAs as well. This repo is very much work in progress.

## How to use
The CLI isn't done yet, but the `dandy-cli` folder is what is going
to become the binary. Change the main function and do `cargo run` to
run it all.

You can also build a (very simple) website which serves to check if
two DFAs denoted in the format is equivalent or not. Building the
website is done with `cd dandy-wasm` and `./build.sh`, and the output
website is located in `web-build`.

## Features
The only feature that is available now is parsing a DFA and checking
if two DFAs are equivalent or not.