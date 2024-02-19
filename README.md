# Wheat

Wheat is a CHIP-8 emulator written in Rust using SDL 2.

## Introduction

CHIP-8 is an interpreted programming language that was designed for the COSMAC VIP. It is commonly seen as the "hello world" of emulators. CHIP-8 has 16 general purpose registers, two timers, 16 input keys, and a monochrome display.

The name comes from the French word for 8 ("huit"). "Wheat" is an English approximation for "huit".

## Features

Wheat passes the full [Timendus test suite](https://github.com/Timendus/chip8-test-suite), and it can play CHIP 8 games. Note: only CHIP-8 is supported. None of the extensions (SUPER-CHIP, XO-CHIP, etc) are implemented.

Key map:

| Keys   | Keys   | Keys   | Keys   |
|--------|--------|--------|--------|
| 1 (0x1) | 2 (0x2) | 3 (0x3) | 4 (0xC) |
| Q (0x4) | W (0x5) | E (0x6) | R (0xD) |
| A (0x7) | S (0x8) | D (0x9) | F (0xE) |
| Z (0xA) | X (0x0) | C (0xB) | V (0xF) |

## Building

Run `cargo build` to build the project, and `cargo run <path-to-ROM>` to launch a game.

## Configuration

The default quirks/settings for Wheat are set to the quirks that the original CHIP-8 shipped with. There are several options below that allow changing these quirks; some games rely on different quirks to function properly. For example, Space Invaders relies on `--q-use-vy-in-shift` being set to `false`. Otherwise, enemies can disappear randomly. More quirks can be found [here](https://chip8.gulrak.net/).

```txt
Options:
  -f, --freq-cpu <FREQ_CPU>
          Frequency (in Hz) for the Chip 8 CPU to run at [default: 800]
      --freq-input <FREQ_INPUT>
          Frequency (in Hz) for the input system to scan new keycodes [default: 12]
      --freq-timer <FREQ_TIMER>
          Frequency (in Hz) for the timers. It is not recommended to change it from the default value [default: 60]
      --q-reset-vf <Q_RESET_VF>
          Quirk: hould the `AND`, `OR`, and `XOR` instructions reset the `VF` register? [default: true] [possible values: true, false]
      --q-increment-ir <Q_INCREMENT_IR>
          Quirk: should the `Fx55` and `Fx65` opcodes increment the index register? Games from the 1970s and 1980s might rely on it being incremented. Modern games might rely on it not being incremented [default: true] [possible values: true, false]
      --q-use-vy-in-shift <Q_USE_VY_IN_SHIFT>
          Quirk: should register `VX` be set to the value of register `VY` before shifting? Modern games might require this to be false [default: true] [possible values: true, false]
      --q-use-vx-in-jump <Q_USE_VX_IN_JUMP>
          Quirk: allow using registers in `0xBnnn` instruction? Interprets `0xB` instructions as `0xBXnn`, where `X` is the register to use as part of the jump, i.e. `VX + nn` instead of `V0 + nnn` [default: false] [possible values: true, false]
      --q-clipping <Q_CLIPPING>
          Quirk: clip the drawings that extend past the screen? Otherwise wraps them and draws them on the other side [default: true] [possible values: true, false]
      --print-opcodes <PRINT_OPCODES>
          Print opcodes as they're interpreted [default: false] [possible values: true, false]
      --dump-graphics <DUMP_GRAPHICS>
          Dump the graphics buffer after every draw opcode [default: false] [possible values: true, false]
  -h, --help
          Print help
  -V, --version
          Print version
```
