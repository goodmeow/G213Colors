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

#[derive(Default)]
struct CommandCaptures {
    field: Option<u32>,
    color: Option<Rgb>,
    speed_ms: Option<u32>,
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

pub fn validate_command_for_spec(spec: &DeviceSpec, command: &str) -> Result<()> {
    if command.is_empty() {
        return Err(invalid_command(command, "command is empty"));
    }

    if command.len() % 2 != 0 || !command.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid_command(command, "command must be even-length hex"));
    }

    let templates = [spec.color_command, spec.breathe_command, spec.cycle_command];
    for template in templates {
        if command_matches_template(spec, command, template)? {
            return Ok(());
        }
    }

    Err(invalid_command(command, "unsupported command form"))
}

pub fn effect_from_commands(spec: &DeviceSpec, commands: &[String]) -> Result<Option<Effect>> {
    if commands.is_empty() {
        return Ok(None);
    }

    for command in commands {
        validate_command_for_spec(spec, command)?;
    }

    if commands.len() == 1 {
        let command = &commands[0];

        if let Some(captures) = capture_command_template(spec, command, spec.color_command)? {
            if captures.field == Some(0) {
                if let Some(color) = captures.color {
                    return Ok(Some(Effect::Static(color)));
                }
            }
        }

        if let Some(captures) = capture_command_template(spec, command, spec.breathe_command)? {
            if let (Some(color), Some(speed_ms)) = (captures.color, captures.speed_ms) {
                return Ok(Some(Effect::Breathe { color, speed_ms }));
            }
        }

        if let Some(captures) = capture_command_template(spec, command, spec.cycle_command)? {
            if let Some(speed_ms) = captures.speed_ms {
                return Ok(Some(Effect::Cycle { speed_ms }));
            }
        }
    }

    Ok(segment_colors_from_commands(spec, commands).map(Effect::Segments))
}

pub fn segment_commands_with_update(
    spec: &DeviceSpec,
    existing_commands: Option<&[String]>,
    segment: u8,
    color: Rgb,
) -> Result<Vec<String>> {
    validate_segment(spec, segment)?;
    let mut colors = existing_commands
        .and_then(|commands| segment_colors_from_commands(spec, commands))
        .unwrap_or([Rgb::WHITE; 5]);
    colors[usize::from(segment - 1)] = color;
    commands_for_effect(spec, &Effect::Segments(colors))
}

fn color_command(spec: &DeviceSpec, field: u8, color: Rgb) -> String {
    spec.color_command
        .replace("{field}", &format!("{field:02x}"))
        .replace("{color}", &color.to_hex())
}

fn command_matches_template(spec: &DeviceSpec, command: &str, template: &str) -> Result<bool> {
    Ok(capture_command_template(spec, command, template)?.is_some())
}

fn capture_command_template(
    spec: &DeviceSpec,
    command: &str,
    template: &str,
) -> Result<Option<CommandCaptures>> {
    let mut command_index = 0;
    let mut template_index = 0;
    let mut captures = CommandCaptures::default();

    while template_index < template.len() {
        let rest = &template[template_index..];
        if rest.starts_with("{field}") {
            let Some(value) = parse_command_hex(command, command_index, 2) else {
                return Ok(None);
            };
            if value > u32::from(spec.zone_count()) {
                return Ok(None);
            }
            captures.field = Some(value);
            command_index += 2;
            template_index += "{field}".len();
        } else if rest.starts_with("{color}") {
            let Some(value) = command.get(command_index..command_index + 6) else {
                return Ok(None);
            };
            let Ok(color) = Rgb::parse_hex(value) else {
                return Ok(None);
            };
            captures.color = Some(color);
            command_index += 6;
            template_index += "{color}".len();
        } else if rest.starts_with("{speed}") {
            let Some(value) = parse_command_hex(command, command_index, 4) else {
                return Ok(None);
            };
            validate_speed(value)?;
            captures.speed_ms = Some(value);
            command_index += 4;
            template_index += "{speed}".len();
        } else {
            if command_index >= command.len() {
                return Ok(None);
            }

            let expected = template.as_bytes()[template_index];
            let actual = command.as_bytes()[command_index];
            if !actual.eq_ignore_ascii_case(&expected) {
                return Ok(None);
            }
            command_index += 1;
            template_index += 1;
        }
    }

    Ok((command_index == command.len()).then_some(captures))
}

fn segment_colors_from_commands(spec: &DeviceSpec, commands: &[String]) -> Option<[Rgb; 5]> {
    let mut colors = [Rgb::WHITE; 5];
    let prefix = spec
        .color_command
        .split("{field}")
        .next()?
        .to_ascii_lowercase();
    let mut found_segment = false;

    for command in commands {
        validate_command_for_spec(spec, command).ok()?;
        let command = command.to_ascii_lowercase();
        if !command.starts_with(&prefix) {
            continue;
        }

        let field_start = prefix.len();
        let field = parse_command_hex(&command, field_start, 2)? as u8;
        if !(1..=spec.zone_count()).contains(&field) {
            continue;
        }

        let mode_start = field_start + 2;
        if !command[mode_start..].starts_with("01") {
            continue;
        }

        let color_start = mode_start + 2;
        let red = parse_command_hex(&command, color_start, 2)? as u8;
        let green = parse_command_hex(&command, color_start + 2, 2)? as u8;
        let blue = parse_command_hex(&command, color_start + 4, 2)? as u8;
        colors[usize::from(field - 1)] = Rgb { red, green, blue };
        found_segment = true;
    }

    found_segment.then_some(colors)
}

fn parse_command_hex(command: &str, start: usize, width: usize) -> Option<u32> {
    let end = start.checked_add(width)?;
    let value = command.get(start..end)?;
    u32::from_str_radix(value, 16).ok()
}

fn invalid_command(command: &str, reason: &'static str) -> G213Error {
    G213Error::InvalidCommand {
        command: command.to_string(),
        reason,
    }
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

    #[test]
    fn validates_only_supported_command_forms() {
        assert!(
            validate_command_for_spec(&G213_SPEC, "11ff0c3a0001ffffff0200000000000000000000")
                .is_ok()
        );
        assert!(validate_command_for_spec(&G213_SPEC, "00").is_err());
        assert!(
            validate_command_for_spec(&G213_SPEC, "11ff0c3a00010102030405060708090a0b0c0d")
                .is_err()
        );
    }

    #[test]
    fn restores_static_effect_from_saved_command() {
        let commands = commands_for_effect(
            &G213_SPEC,
            &Effect::Static(Rgb::parse_hex("12ab34").unwrap()),
        )
        .unwrap();

        assert_eq!(
            effect_from_commands(&G213_SPEC, &commands).unwrap(),
            Some(Effect::Static(Rgb::parse_hex("12ab34").unwrap()))
        );
    }

    #[test]
    fn restores_breathe_effect_from_saved_command() {
        let commands = commands_for_effect(
            &G213_SPEC,
            &Effect::Breathe {
                color: Rgb::parse_hex("00ff00").unwrap(),
                speed_ms: 5000,
            },
        )
        .unwrap();

        assert_eq!(
            effect_from_commands(&G213_SPEC, &commands).unwrap(),
            Some(Effect::Breathe {
                color: Rgb::parse_hex("00ff00").unwrap(),
                speed_ms: 5000,
            })
        );
    }

    #[test]
    fn restores_cycle_effect_from_saved_command() {
        let commands = commands_for_effect(&G213_SPEC, &Effect::Cycle { speed_ms: 5000 }).unwrap();

        assert_eq!(
            effect_from_commands(&G213_SPEC, &commands).unwrap(),
            Some(Effect::Cycle { speed_ms: 5000 })
        );
    }

    #[test]
    fn segment_update_preserves_existing_segment_colors() {
        let existing = commands_for_effect(
            &G213_SPEC,
            &Effect::Segments([
                Rgb::parse_hex("111111").unwrap(),
                Rgb::parse_hex("222222").unwrap(),
                Rgb::parse_hex("333333").unwrap(),
                Rgb::parse_hex("444444").unwrap(),
                Rgb::parse_hex("555555").unwrap(),
            ]),
        )
        .unwrap();

        let commands = segment_commands_with_update(
            &G213_SPEC,
            Some(&existing),
            3,
            Rgb::parse_hex("abcdef").unwrap(),
        )
        .unwrap();

        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], "11ff0c3a01011111110200000000000000000000");
        assert_eq!(commands[2], "11ff0c3a0301abcdef0200000000000000000000");
        assert_eq!(commands[4], "11ff0c3a05015555550200000000000000000000");
    }

    #[test]
    fn segment_update_preserves_partial_uppercase_segment_config() {
        let existing = vec!["11FF0C3A02012222220200000000000000000000".to_string()];

        let commands = segment_commands_with_update(
            &G213_SPEC,
            Some(&existing),
            3,
            Rgb::parse_hex("abcdef").unwrap(),
        )
        .unwrap();

        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], "11ff0c3a0101ffffff0200000000000000000000");
        assert_eq!(commands[1], "11ff0c3a02012222220200000000000000000000");
        assert_eq!(commands[2], "11ff0c3a0301abcdef0200000000000000000000");
    }

    #[test]
    fn restores_segment_effect_from_saved_commands() {
        let colors = [
            Rgb::parse_hex("111111").unwrap(),
            Rgb::parse_hex("222222").unwrap(),
            Rgb::parse_hex("333333").unwrap(),
            Rgb::parse_hex("444444").unwrap(),
            Rgb::parse_hex("555555").unwrap(),
        ];
        let commands = commands_for_effect(&G213_SPEC, &Effect::Segments(colors)).unwrap();

        assert_eq!(
            effect_from_commands(&G213_SPEC, &commands).unwrap(),
            Some(Effect::Segments(colors))
        );
    }
}
