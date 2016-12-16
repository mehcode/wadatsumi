# Wadatsumi

## Build

```sh
$ cargo build --release
```

## Install

```sh
$ cargo install
```

## Usage

```sh
$ wadatsumi [OPTIONS] <rom>
```

## Options

### Mode

```
-m <device>[:<variation>]
```

Mode is specified as `-m <device>:<variation>` with `:<variation>` able to
be omitted (which will default to a common variation). The default variation
is marked with `(*)` in the following table. If mode is left unspecified;
wadatsumi, will attempt to guess the preferred mode.

| Device                     | Variation  | Notes                              |
| -------------------------- | ---------- | ---------------------------------- |
| Gameboy — `gb`             | `dmg0`     |                                    |
| Gameboy — `gb`             | `dmg`      |                                    |
| Gameboy — `gb`             | `mgb`  (*) |                                    |
| Gameboy — `gb`             | `cgb`      | CGB locked in GB mode              |
| Gameboy — `gb`             | `agb`      | AGB locked in GB mode              |
| Gameboy — `gb`             | `sgb1`     | SGB1 locked in GB mode             |
| Gameboy — `gb`             | `sgb2`     | SGB2 locked in GB mode             |
| Super Gameboy — `sgb`      | `1`        |                                    |
| Super Gameboy — `sgb`      | `2`    (*) |                                    |
| Color Gameboy — `cgb`      | `cgb`  (*) |                                    |
| Color Gameboy — `cgb`      | `agb`      | AGB locked in CGB mode             |

### DMG/MGB Palette

Applicable for modes: `gb:dmg0`, `gb:dmg`, `gb:mgb`

```
-Z gb:palette=<0>,<1>,<2>,<3>
```

Each color in the palette is a hex code in the format: `RRGGBB`.

A 4-shade palette that resembles the original gameboy may be explicitly
specified with `-Z gb:palette=9BBC0F,8BB30F,306230,0F410F`.
