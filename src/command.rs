use crate::error::{G213Error, Result};
use crate::product::DeviceSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub const WHITE: Self = Self {
        red: 255,
        green: 255,
        blue: 255,
    };

    pub fn parse_hex(input: &str) -> Result<Self> {
        if input.len() != 6 || !input.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(G213Error::InvalidColor(input.to_string()));
        }

        let red = u8::from_str_radix(&input[0..2], 16)
            .map_err(|_| G213Error::InvalidColor(input.to_string()))?;
        let green = u8::from_str_radix(&input[2..4], 16)
            .map_err(|_| G213Error::InvalidColor(input.to_string()))?;
        let blue = u8::from_str_radix(&input[4..6], 16)
            .map_err(|_| G213Error::InvalidColor(input.to_string()))?;
        Ok(Self { red, green, blue })
    }

    pub fn to_hex(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Static(Rgb),
    Breathe { color: Rgb, speed_ms: u32 },
    Cycle { speed_ms: u32 },
    Segments([Rgb; 5]),
}

pub fn validate_speed(speed_ms: u32) -> Result<u32> {
    if !(500..=65_535).contains(&speed_ms) {
        return Err(G213Error::InvalidSpeed(speed_ms));
    }
    Ok(speed_ms)
}

pub fn validate_segment(spec: &DeviceSpec, segment: u8) -> Result<u8> {
    let max = spec.zone_count();
    if !(1..=max).contains(&segment) {
        return Err(G213Error::InvalidSegment { segment, max });
    }
    Ok(segment)
}

pub fn static_command(spec: &DeviceSpec, color: Rgb) -> String {
    color_command(spec, 0, color)
}

pub fn segment_command(spec: &DeviceSpec, segment: u8, color: Rgb) -> Result<String> {
    validate_segment(spec, segment)?;
    Ok(color_command(spec, segment, color))
}

pub fn breathe_command(spec: &DeviceSpec, color: Rgb, speed_ms: u32) -> Result<String> {
    let speed_ms = validate_speed(speed_ms)?;
    Ok(spec
        .breathe_command
        .replace("{color}", &color.to_hex())
        .replace("{speed}", &format!("{speed_ms:04x}")))
}

pub fn cycle_command(spec: &DeviceSpec, speed_ms: u32) -> Result<String> {
    let speed_ms = validate_speed(speed_ms)?;
    Ok(spec
        .cycle_command
        .replace("{speed}", &format!("{speed_ms:04x}")))
}

pub fn commands_for_effect(spec: &DeviceSpec, effect: &Effect) -> Result<Vec<String>> {
    match effect {
        Effect::Static(color) => Ok(vec![static_command(spec, *color)]),
        Effect::Breathe { color, speed_ms } => Ok(vec![breathe_command(spec, *color, *speed_ms)?]),
        Effect::Cycle { speed_ms } => Ok(vec![cycle_command(spec, *speed_ms)?]),
        Effect::Segments(colors) => colors
            .iter()
            .enumerate()
            .map(|(index, color)| segment_command(spec, index as u8 + 1, *color))
            .collect(),
    }
}

fn color_command(spec: &DeviceSpec, field: u8, color: Rgb) -> String {
    spec.color_command
        .replace("{field}", &format!("{field:02x}"))
        .replace("{color}", &color.to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::G213_SPEC;

    #[test]
    fn parses_hex_color() {
        assert_eq!(
            Rgb::parse_hex("ff00aa").unwrap(),
            Rgb {
                red: 255,
                green: 0,
                blue: 170
            }
        );
        assert!(Rgb::parse_hex("xyz").is_err());
        assert!(Rgb::parse_hex("ff00aax").is_err());
    }

    #[test]
    fn validates_speed_range() {
        assert!(validate_speed(500).is_ok());
        assert!(validate_speed(65_535).is_ok());
        assert!(validate_speed(499).is_err());
        assert!(validate_speed(65_536).is_err());
    }

    #[test]
    fn builds_g213_static_command() {
        assert_eq!(
            static_command(&G213_SPEC, Rgb::parse_hex("ffffff").unwrap()),
            "11ff0c3a0001ffffff0200000000000000000000"
        );
    }

    #[test]
    fn builds_g213_breathe_command() {
        assert_eq!(
            breathe_command(&G213_SPEC, Rgb::parse_hex("00ff00").unwrap(), 5000).unwrap(),
            "11ff0c3a000200ff001388006400000000000000"
        );
    }

    #[test]
    fn builds_g213_cycle_command() {
        assert_eq!(
            cycle_command(&G213_SPEC, 5000).unwrap(),
            "11ff0c3a0003ffffff0000138864000000000000"
        );
    }

    #[test]
    fn builds_g213_segment_command() {
        assert_eq!(
            segment_command(&G213_SPEC, 5, Rgb::parse_hex("123456").unwrap()).unwrap(),
            "11ff0c3a05011234560200000000000000000000"
        );
        assert!(segment_command(&G213_SPEC, 6, Rgb::WHITE).is_err());
    }
}
