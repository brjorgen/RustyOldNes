
# Table of Contents

1.  [RustyOldNes](#org3ea6173)
2.  [Building](#org3102960)
3.  [Usage](#orgaaa200a)



<a id="org3ea6173"></a>

# RustyOldNes

An emulator for the Nintendo Entertainment System, written in Rust.

Not quite "Feed it your favourite NES Rom and enjoy!" yet, as most 6502
instructions, the APU and PPU have yet to be impemented.


<a id="org3102960"></a>

# Building

    git clone https://github.com/brjorgen/RustyOldNes.git
    cd RustyOldNes
    cargo build --release


<a id="orgaaa200a"></a>

# Usage

    cargo run [path/to/your/rom]

or

    ./target/release/rusty_old_nes

