use std::vec::Vec;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
pub enum Device {
    GB,
    CGB,
    SGB,
}

pub use self::Device::*;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    GB_DMG0,
    GB_DMG,
    GB_MGB,
    GB_CGB,
    GB_AGB,
    GB_SGB1,
    GB_SGB2,

    CGB_CGB,
    CGB_AGB,

    SGB_SGB1,
    SGB_SGB2,
}

pub use self::Mode::*;

impl Mode {
    pub fn from_str(s: &str) -> Option<Mode> {
        // Split mode str on `:`
        let v: Vec<&str> = s.split(':').collect();

        // Determine device
        let device = match v[0] {
            "gb" => GB,
            "cgb" => CGB,
            "sgb" => SGB,
            _ => {
                // Unknown device
                return None;
            }
        };

        Some(if v.len() == 1 {
            // Default mode from device
            match device {
                GB => GB_MGB,
                CGB => CGB_CGB,
                SGB => SGB_SGB2,
            }
        } else {
            // Determine mode from device:variation
            match (device, v[1]) {
                (GB, "dmg0") => GB_DMG0,
                (GB, "dmg") => GB_DMG,
                (GB, "mgb") => GB_MGB,
                (GB, "cgb") => GB_CGB,
                (GB, "agb") => GB_AGB,
                (GB, "sgb1") => GB_SGB1,
                (GB, "sgb") | (GB, "sgb2") => GB_SGB2,

                (CGB, "cgb") => CGB_CGB,
                (CGB, "agb") => CGB_AGB,

                (SGB, "1") => SGB_SGB1,
                (SGB, "2") => SGB_SGB2,

                _ => {
                    // Unknown mode
                    return None;
                }
            }
        })
    }

    /// Query device from mode
    pub fn device(&self) -> Device {
        match *self {
            GB_DMG0 | GB_DMG | GB_MGB | GB_CGB | GB_AGB | GB_SGB1 | GB_SGB2 => GB,
            CGB_CGB | CGB_AGB => CGB,
            SGB_SGB1 | SGB_SGB2 => SGB,
        }
    }
}
