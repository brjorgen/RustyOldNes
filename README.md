
# Table of Contents

1.  [RustyOldNes](#org3b7fcb5)
2.  [Building](#orgcda790f)
3.  [Usage](#org372d615)



<a id="org3b7fcb5"></a>

# RustyOldNes

[![Rust](![img](https://github.com/brjorgen/RustyOldNes/actions/workflows/rust.yml/badge.svg))](<https://github.com/brjorgen/RustyOldNes/actions/workflows/rust.yml>)
An emulator for the Nintendo Entertainment System, written in Rust.

Not quite "Feed it your favourite NES Rom and enjoy!" yet, as most 6502
instructions, the APU and PPU have yet to be impemented.


<a id="orgcda790f"></a>

# Building

    git clone https://github.com/brjorgen/RustyOldNes.git
    cd RustyOldNes
    cargo build --release


<a id="org372d615"></a>

# Usage

    cargo run [path/to/your/rom]

or

    ./target/release/rusty_old_nes [path/to/your/rom]

