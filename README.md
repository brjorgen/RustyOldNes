
# Table of Contents

1.  [RustyOldNes](#org57cb1e5)
2.  [Building](#orge0f5b76)
3.  [Usage](#org5740820)



<a id="org57cb1e5"></a>

# RustyOldNes

An emulator for the Nintendo Entertainment System, written in Rust.

Not quite "Feed it your favourite NES Rom and enjoy!" yet, as most 6502
instructions, the APU and PPU have yet to be impemented.


<a id="orge0f5b76"></a>

# Building

    git clone https://github.com/brjorgen/RustyOldNes.git
    cd RustyOldNes
    cargo build --release


<a id="org5740820"></a>

# Usage

    cargo run [path/to/your/rom]

or

    ./target/release/rusty_old_nes [path/to/your/rom]

