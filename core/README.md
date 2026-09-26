# Chippy
Chippy is a Rust-based emulator, debugger, and library that supports 
CHIP-8, SChip 1.x, and XO-Chip as well as many of the quirks 
that exist on these systems.


# Installation
## As a TUI
This repo ships a basic TUI that can be installed with `cargo install --git "https://github.com/Muu79/chippy"`.
This will install the TUI as `chippy`

### Usage
Basic usage can be done as `chippy /path/to/rom <target>`, the current supported targets are:

- `scleg`: Super Chip 1.x with legacy quirks for physical hardware support on the HP48
- `scmod`: Super Chip 1.x with modern quirks consistent with newer implementations of SChip
- `xoc`: XO-Chip support as per the official [reference](https://johnearnest.github.io/Octo/docs/XO-ChipSpecification.html)
- Omitting target or any other input not listed above will default to plain CHIP-8

## As a library
This repo also ships a chippy_core crate that provides a backend for use with any frontend.