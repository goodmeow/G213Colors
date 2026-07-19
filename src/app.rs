use std::fmt;

use iced::widget::{button, checkbox, column, container, pick_list, row, rule, slider, text};
use iced::{Element, Length, Task};

use crate::command::{Effect, Rgb};
use crate::config;
use crate::device;
use crate::product::Product;

const APP_TITLE: &str = "G213 Colors";
const EFFECT_TABS: [EffectTab; 4] = [
    EffectTab::Static,
    EffectTab::Cycle,
    EffectTab::Breathe,
    EffectTab::Segments,
];

pub fn run() -> iced::Result {
    iced::application(G213App::new, G213App::update, G213App::view)
        .title(app_title)
        .run()
}

fn app_title(_: &G213App) -> String {
    APP_TITLE.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectTab {
    Static,
    Cycle,
    Breathe,
    Segments,
}

impl fmt::Display for EffectTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static => f.write_str("Static"),
            Self::Cycle => f.write_str("Cycle"),
            Self::Breathe => f.write_str("Breathe"),
            Self::Segments => f.write_str("Segments"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    EffectSelected(EffectTab),
    StaticRedChanged(u8),
    StaticGreenChanged(u8),
    StaticBlueChanged(u8),
    BreatheRedChanged(u8),
    BreatheGreenChanged(u8),
    BreatheBlueChanged(u8),
    CycleSpeedChanged(u32),
    BreatheSpeedChanged(u32),
    SegmentRedChanged(usize, u8),
    SegmentGreenChanged(usize, u8),
    SegmentBlueChanged(usize, u8),
    ScanPressed,
    ScanFinished(bool),
    ApplyPressed,
    ApplyFinished(std::result::Result<(), String>),
    AutostartToggled(bool),
    AutostartFinished(bool, std::result::Result<(), String>),
}

struct G213App {
    selected_effect: EffectTab,
    static_color: Rgb,
    breathe_color: Rgb,
    cycle_speed_ms: u32,
    breathe_speed_ms: u32,
    segment_colors: [Rgb; 5],
    connected_g213: bool,
    autostart_enabled: bool,
    autostart_busy: bool,
    busy: bool,
    status: String,
}

impl G213App {
    fn new() -> (Self, Task<Message>) {
        let status = match config::ensure_user_dirs() {
            Ok(()) => "Ready. Scan G213 to begin.".to_string(),
            Err(error) => format!("Config directory warning: {error}"),
        };

        (
            Self {
                selected_effect: EffectTab::Static,
                static_color: Rgb::WHITE,
                breathe_color: Rgb::WHITE,
                cycle_speed_ms: 5000,
                breathe_speed_ms: 5000,
                segment_colors: [Rgb::WHITE; 5],
                connected_g213: false,
                autostart_enabled: config::is_autostart_enabled(Product::G213),
                autostart_busy: false,
                busy: true,
                status,
            },
            Task::perform(
                async { device::detect(Product::G213) },
                Message::ScanFinished,
            ),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EffectSelected(effect) => {
                self.selected_effect = effect;
                Task::none()
            }
            Message::StaticRedChanged(value) => {
                self.static_color.red = value;
                Task::none()
            }
            Message::StaticGreenChanged(value) => {
                self.static_color.green = value;
                Task::none()
            }
            Message::StaticBlueChanged(value) => {
                self.static_color.blue = value;
                Task::none()
            }
            Message::BreatheRedChanged(value) => {
                self.breathe_color.red = value;
                Task::none()
            }
            Message::BreatheGreenChanged(value) => {
                self.breathe_color.green = value;
                Task::none()
            }
            Message::BreatheBlueChanged(value) => {
                self.breathe_color.blue = value;
                Task::none()
            }
            Message::CycleSpeedChanged(value) => {
                self.cycle_speed_ms = value;
                Task::none()
            }
            Message::BreatheSpeedChanged(value) => {
                self.breathe_speed_ms = value;
                Task::none()
            }
            Message::SegmentRedChanged(index, value) => {
                self.segment_colors[index].red = value;
                Task::none()
            }
            Message::SegmentGreenChanged(index, value) => {
                self.segment_colors[index].green = value;
                Task::none()
            }
            Message::SegmentBlueChanged(index, value) => {
                self.segment_colors[index].blue = value;
                Task::none()
            }
            Message::ScanPressed => {
                self.busy = true;
                self.status = "Scanning for G213...".to_string();
                Task::perform(
                    async { device::detect(Product::G213) },
                    Message::ScanFinished,
                )
            }
            Message::ScanFinished(found) => {
                self.busy = false;
                self.connected_g213 = found;
                self.status = if found {
                    "G213 detected.".to_string()
                } else {
                    "No G213 detected.".to_string()
                };
                Task::none()
            }
            Message::ApplyPressed => {
                if !self.connected_g213 {
                    self.status = "Scan did not find a connected G213.".to_string();
                    return Task::none();
                }

                let effect = self.current_effect();
                self.busy = true;
                self.status = format!("Applying {} settings...", self.selected_effect);
                Task::perform(
                    async move {
                        device::apply_effect(Product::G213, &effect)
                            .map_err(|error| error.to_string())
                    },
                    Message::ApplyFinished,
                )
            }
            Message::ApplyFinished(result) => {
                self.busy = false;
                self.status = match result {
                    Ok(()) => "G213 settings applied and saved.".to_string(),
                    Err(error) => format!("Apply failed: {error}"),
                };
                Task::none()
            }
            Message::AutostartToggled(enabled) => {
                if self.autostart_busy {
                    return Task::none();
                }

                self.autostart_enabled = enabled;
                self.autostart_busy = true;
                self.status = if enabled {
                    "Creating G213 autostart entry...".to_string()
                } else {
                    "Removing G213 autostart entry...".to_string()
                };
                Task::perform(
                    async move {
                        let result = if enabled {
                            config::create_autostart_entry(Product::G213)
                        } else {
                            config::remove_autostart_entry(Product::G213)
                        };
                        result.map_err(|error| error.to_string())
                    },
                    move |result| Message::AutostartFinished(enabled, result),
                )
            }
            Message::AutostartFinished(enabled, result) => {
                self.autostart_busy = false;
                match result {
                    Ok(()) => {
                        self.autostart_enabled = enabled;
                        self.status = if enabled {
                            "G213 autostart enabled.".to_string()
                        } else {
                            "G213 autostart disabled.".to_string()
                        };
                    }
                    Err(error) => {
                        self.autostart_enabled = !enabled;
                        self.status = format!("Autostart update failed: {error}");
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let scan_button = if self.busy {
            button("Scan G213")
        } else {
            button("Scan G213").on_press(Message::ScanPressed)
        };

        let set_button = if self.connected_g213 && !self.busy {
            button("Set G213").on_press(Message::ApplyPressed)
        } else {
            button("Set G213")
        };

        let header = column![
            text(APP_TITLE).size(28),
            text("Logitech G213 lighting controller").size(16),
            row![
                scan_button,
                set_button,
                text(if self.connected_g213 {
                    "Connected"
                } else {
                    "Not detected"
                })
            ]
            .spacing(12)
        ]
        .spacing(8);

        let effect_picker = pick_list(
            EFFECT_TABS.as_slice(),
            Some(self.selected_effect),
            Message::EffectSelected,
        );

        let effect_controls = match self.selected_effect {
            EffectTab::Static => self.static_controls(),
            EffectTab::Cycle => self.cycle_controls(),
            EffectTab::Breathe => self.breathe_controls(),
            EffectTab::Segments => self.segment_controls(),
        };

        let autostart = checkbox(self.autostart_enabled).label("Apply user settings on login");
        let autostart = if self.autostart_busy {
            autostart
        } else {
            autostart.on_toggle(Message::AutostartToggled)
        };

        container(
            column![
                header,
                rule::horizontal(1),
                row![text("Effect"), effect_picker].spacing(12),
                effect_controls,
                rule::horizontal(1),
                autostart,
                text(&self.status)
            ]
            .spacing(14),
        )
        .padding(18)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn current_effect(&self) -> Effect {
        match self.selected_effect {
            EffectTab::Static => Effect::Static(self.static_color),
            EffectTab::Cycle => Effect::Cycle {
                speed_ms: self.cycle_speed_ms,
            },
            EffectTab::Breathe => Effect::Breathe {
                color: self.breathe_color,
                speed_ms: self.breathe_speed_ms,
            },
            EffectTab::Segments => Effect::Segments(self.segment_colors),
        }
    }

    fn static_controls(&self) -> Element<'_, Message> {
        column![
            text(format!("Static #{hex}", hex = self.static_color.to_hex())),
            rgb_sliders(
                self.static_color,
                Message::StaticRedChanged,
                Message::StaticGreenChanged,
                Message::StaticBlueChanged,
            )
        ]
        .spacing(10)
        .into()
    }

    fn cycle_controls(&self) -> Element<'_, Message> {
        column![
            text(format!("Speed: {} ms", self.cycle_speed_ms)),
            slider(
                500..=65_535,
                self.cycle_speed_ms,
                Message::CycleSpeedChanged
            )
        ]
        .spacing(10)
        .into()
    }

    fn breathe_controls(&self) -> Element<'_, Message> {
        column![
            text(format!("Breathe #{hex}", hex = self.breathe_color.to_hex())),
            rgb_sliders(
                self.breathe_color,
                Message::BreatheRedChanged,
                Message::BreatheGreenChanged,
                Message::BreatheBlueChanged,
            ),
            text(format!("Speed: {} ms", self.breathe_speed_ms)),
            slider(
                500..=65_535,
                self.breathe_speed_ms,
                Message::BreatheSpeedChanged
            )
        ]
        .spacing(10)
        .into()
    }

    fn segment_controls(&self) -> Element<'_, Message> {
        let mut controls = column![].spacing(12);
        for (index, color) in self.segment_colors.iter().copied().enumerate() {
            controls = controls.push(
                column![
                    text(format!(
                        "Segment {} #{hex}",
                        index + 1,
                        hex = color.to_hex()
                    )),
                    rgb_sliders(
                        color,
                        move |value| Message::SegmentRedChanged(index, value),
                        move |value| Message::SegmentGreenChanged(index, value),
                        move |value| Message::SegmentBlueChanged(index, value),
                    )
                ]
                .spacing(6),
            );
        }
        controls.into()
    }
}

fn rgb_sliders<'a>(
    color: Rgb,
    red_message: impl Fn(u8) -> Message + 'a,
    green_message: impl Fn(u8) -> Message + 'a,
    blue_message: impl Fn(u8) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        row![
            text("R").width(20),
            slider(0..=255, color.red, red_message),
            text(color.red)
        ]
        .spacing(8),
        row![
            text("G").width(20),
            slider(0..=255, color.green, green_message),
            text(color.green)
        ]
        .spacing(8),
        row![
            text("B").width(20),
            slider(0..=255, color.blue, blue_message),
            text(color.blue)
        ]
        .spacing(8)
    ]
    .spacing(6)
    .into()
}
