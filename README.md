# Wadatsumi

## Usage

### Mode

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
