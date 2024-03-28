# Lul

Lul is a lightweight command runner, inspired by tools like
[`just`](https://github.com/casey/just) and if you squint hard enough, perhaps
GNU Make.

Like `just`, `lul` is a *command runner*, not a build tool, but unlike it, it
uses an embedded Lua runtime, so what you get is a full-blown Lua (actually,
[Luau](https://luau-lang.org/)) interpreter.

At its core, `lul` aims to be uncomplicated and stay out of your way: If you
already know Lua, there's very little to learn: When invoked as `lul <taskname>`
`lul` will locate the nearest `lulfile`[1] will load it as a lua script and
invoke the function called `<taskname>` in it. Simple!

The other thing `lul` does for you is providing a set of builtin functions that
will make your life easier when writing scripts. This list is currently evolving
so check the source code in `lua/prelude.lua` for a quick summary.

[1]: `Lulfile`, `lulfile.lua` and `.lulfile` are also accepted.

## Installation

Welcome lurker! You've just witnessed the birth of this tool, and thus, there
are no binaries ready.

If you insist, you'll have to get your hands dirty with `cargo`, the rust
build tool:

- Install a Rust compiler toolchain (visit [rustup.rs](https://rustup.rs) for
  instructions)
- Clone this repository
```bash
git clone https://github.com/setzer22/lul
```
- Build the project
```bash
cd lul
cargo build --release
```
- Copy the resulting binary wherever you want, you'll find it under
  `target/release/lul`

## Why Luau and not Lua?

The reason Luau was chosen is that it supports string interpolation, something
that is invaluable during scripts:

This:
```lua
sh("tar -czvf" .. buildsdir .. " -C " .. publishdir .. " .")
```
Becomes this:
```lua
sh(`tar -czvf {buildsdir} -C {publishdir} .`) 
```
*Did you spot the bug in that first version? :)*

## Disclaimer

Like most open source software, this tool scratches an itch, and the time I can
devote to its maintenance beyond supporting my use case is limited. I am sharing
this in hope it can be useful to others in its current state, but please
understand I am unable to fulfill feature requests and review external
contributor PRs in general.