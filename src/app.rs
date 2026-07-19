use std::fmt;

use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable, slider, space, text,
};
use iced::{alignment, border, Color, Element, Length, Task, Theme};

use crate::command::{effect_from_commands, Effect, Rgb};
use crate::config;
use crate::device;
use crate::error::G213Error;
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
        .theme(app_theme)
        .window_size([640.0, 740.0])
        .run()
}

fn app_title(_: &G213App) -> String {
    APP_TITLE.to_string()
}

fn app_theme(_: &G213App) -> Theme {
    Theme::Dark
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
    RestoreFinished(std::result::Result<(), String>),
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
    startup_restore_effect: Option<Effect>,
}

impl G213App {
    fn new() -> (Self, Task<Message>) {
        let config_ready = config::ensure_user_dirs();
        let status = match &config_ready {
            Ok(()) => "Ready. Scan G213 to begin.".to_string(),
            Err(error) => format!("Config directory warning: {error}"),
        };

        let mut app = Self {
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
            startup_restore_effect: None,
        };

        if config_ready.is_ok() {
            match load_saved_effect() {
                Ok(Some(effect)) => {
                    app.set_effect(effect.clone());
                    app.startup_restore_effect = Some(effect);
                    app.status = "Loaded saved G213 settings. Scanning for G213...".to_string();
                }
                Ok(None) => {}
                Err(error) => {
                    app.status = format!("Saved settings warning: {error}");
                }
            }
        }

        (
            app,
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
                if found {
                    if let Some(effect) = self.startup_restore_effect.take() {
                        self.busy = true;
                        self.status = "Restoring saved G213 settings...".to_string();
                        return Task::perform(
                            async move {
                                device::apply_effect(Product::G213, &effect)
                                    .map_err(|error| error.to_string())
                            },
                            Message::RestoreFinished,
                        );
                    }
                }

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
            Message::RestoreFinished(result) => {
                self.busy = false;
                self.status = match result {
                    Ok(()) => "Saved G213 settings restored.".to_string(),
                    Err(error) => format!("Restore failed: {error}"),
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

        let header = row![
            column![
                text(APP_TITLE).size(28),
                text(self.effect_summary()).size(14),
            ]
            .spacing(4),
            space().width(Length::Fill),
            status_badge(self.connected_g213, self.busy),
        ]
        .spacing(16)
        .align_y(alignment::Vertical::Center);

        let actions = panel(
            column![
                row![scan_button, set_button, space().width(Length::Fill),]
                    .spacing(12)
                    .align_y(alignment::Vertical::Center),
                text(&self.status).size(14),
            ]
            .spacing(10),
        );

        let effect_picker = pick_list(
            EFFECT_TABS.as_slice(),
            Some(self.selected_effect),
            Message::EffectSelected,
        );

        let effect_selector = panel(
            row![
                text("Effect").size(14).width(Length::Fixed(88.0)),
                effect_picker,
            ]
            .spacing(12)
            .align_y(alignment::Vertical::Center),
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

        let content = column![
            header,
            actions,
            effect_selector,
            panel(effect_controls),
            panel(
                row![
                    autostart,
                    space().width(Length::Fill),
                    autostart_badge(self.autostart_enabled)
                ]
                .spacing(12)
                .align_y(alignment::Vertical::Center),
            ),
        ]
        .spacing(12)
        .max_width(720.0)
        .width(Length::Fill);

        container(scrollable(
            column![content, rule::horizontal(1), text("G213 Prodigy").size(13),].spacing(12),
        ))
        .padding(18)
        .center_x(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(page_style)
        .into()
    }

    fn effect_summary(&self) -> String {
        match self.current_effect() {
            Effect::Static(color) => format!("Static #{hex}", hex = color.to_hex()),
            Effect::Cycle { speed_ms } => format!("Cycle {speed_ms} ms"),
            Effect::Breathe { color, speed_ms } => {
                format!("Breathe #{hex} at {speed_ms} ms", hex = color.to_hex())
            }
            Effect::Segments(colors) => {
                let preview = colors
                    .iter()
                    .map(|color| format!("#{}", color.to_hex()))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("Segments {preview}")
            }
        }
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

    fn set_effect(&mut self, effect: Effect) {
        match effect {
            Effect::Static(color) => {
                self.selected_effect = EffectTab::Static;
                self.static_color = color;
            }
            Effect::Cycle { speed_ms } => {
                self.selected_effect = EffectTab::Cycle;
                self.cycle_speed_ms = speed_ms;
            }
            Effect::Breathe { color, speed_ms } => {
                self.selected_effect = EffectTab::Breathe;
                self.breathe_color = color;
                self.breathe_speed_ms = speed_ms;
            }
            Effect::Segments(colors) => {
                self.selected_effect = EffectTab::Segments;
                self.segment_colors = colors;
            }
        }
    }

    fn static_controls(&self) -> Element<'_, Message> {
        column![
            large_color_preview("Static color", self.static_color),
            rgb_sliders(
                self.static_color,
                Message::StaticRedChanged,
                Message::StaticGreenChanged,
                Message::StaticBlueChanged,
            )
        ]
        .spacing(14)
        .into()
    }

    fn cycle_controls(&self) -> Element<'_, Message> {
        column![
            metric_row("Cycle speed", format!("{} ms", self.cycle_speed_ms)),
            slider(
                500..=65_535,
                self.cycle_speed_ms,
                Message::CycleSpeedChanged
            )
        ]
        .spacing(14)
        .into()
    }

    fn breathe_controls(&self) -> Element<'_, Message> {
        column![
            large_color_preview("Breathe color", self.breathe_color),
            rgb_sliders(
                self.breathe_color,
                Message::BreatheRedChanged,
                Message::BreatheGreenChanged,
                Message::BreatheBlueChanged,
            ),
            metric_row("Breathe speed", format!("{} ms", self.breathe_speed_ms)),
            slider(
                500..=65_535,
                self.breathe_speed_ms,
                Message::BreatheSpeedChanged
            )
        ]
        .spacing(14)
        .into()
    }

    fn segment_controls(&self) -> Element<'_, Message> {
        let mut controls = column![segment_strip(self.segment_colors)].spacing(12);
        for (index, color) in self.segment_colors.iter().copied().enumerate() {
            controls = controls.push(segment_panel(index, color));
        }
        controls.into()
    }
}

fn panel<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(16)
        .width(Length::Fill)
        .style(panel_style)
        .into()
}

fn segment_panel(index: usize, color: Rgb) -> Element<'static, Message> {
    container(
        column![
            color_heading(format!("Segment {}", index + 1), color),
            rgb_sliders(
                color,
                move |value| Message::SegmentRedChanged(index, value),
                move |value| Message::SegmentGreenChanged(index, value),
                move |value| Message::SegmentBlueChanged(index, value),
            )
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill)
    .style(subtle_panel_style)
    .into()
}

fn segment_strip(colors: [Rgb; 5]) -> Element<'static, Message> {
    let mut strip = row![].spacing(6).width(Length::Fill);
    for color in colors {
        strip = strip.push(
            container(text(""))
                .height(16)
                .width(Length::FillPortion(1))
                .style(move |_| color_box_style(color, 3)),
        );
    }

    strip.into()
}

fn large_color_preview(label: &str, color: Rgb) -> Element<'static, Message> {
    row![
        container(text(""))
            .width(72)
            .height(56)
            .style(move |_| color_box_style(color, 6)),
        column![
            text(label.to_string()).size(14),
            text(format!("#{hex}", hex = color.to_hex())).size(22),
        ]
        .spacing(4)
    ]
    .spacing(14)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn color_heading(label: impl Into<String>, color: Rgb) -> Element<'static, Message> {
    let label = label.into();

    row![
        color_swatch(color),
        text(format!("{label} #{hex}", hex = color.to_hex()))
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn color_swatch(color: Rgb) -> Element<'static, Message> {
    container(text(""))
        .width(24)
        .height(24)
        .style(move |_| color_box_style(color, 4))
        .into()
}

fn rgb_sliders<'a>(
    color: Rgb,
    red_message: impl Fn(u8) -> Message + 'a,
    green_message: impl Fn(u8) -> Message + 'a,
    blue_message: impl Fn(u8) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        channel_slider("R", color.red, red_message),
        channel_slider("G", color.green, green_message),
        channel_slider("B", color.blue, blue_message),
    ]
    .spacing(8)
    .into()
}

fn channel_slider<'a>(
    label: &'static str,
    value: u8,
    on_change: impl Fn(u8) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).width(Length::Fixed(24.0)),
        slider(0..=255, value, on_change),
        text(format!("{value:>3}")).width(Length::Fixed(44.0))
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn metric_row(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label).size(14),
        space().width(Length::Fill),
        text(value).size(22)
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn status_badge(connected: bool, busy: bool) -> Element<'static, Message> {
    let (label, color) = if busy {
        ("Working", Color::from_rgb8(218, 165, 32))
    } else if connected {
        ("Connected", Color::from_rgb8(45, 160, 92))
    } else {
        ("Not detected", Color::from_rgb8(150, 155, 165))
    };

    container(text(label).size(13))
        .padding([6, 10])
        .style(move |_| {
            iced::widget::container::Style::default()
                .background(color)
                .color(Color::WHITE)
                .border(border::rounded(4))
        })
        .into()
}

fn autostart_badge(enabled: bool) -> Element<'static, Message> {
    let (label, color) = if enabled {
        ("Enabled", Color::from_rgb8(70, 130, 180))
    } else {
        ("Off", Color::from_rgb8(110, 116, 128))
    };

    container(text(label).size(13))
        .padding([6, 10])
        .style(move |_| {
            iced::widget::container::Style::default()
                .background(color)
                .color(Color::WHITE)
                .border(border::rounded(4))
        })
        .into()
}

fn page_style(_: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style::default().background(Color::from_rgb8(22, 24, 28))
}

fn panel_style(_: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style::default()
        .background(Color::from_rgb8(34, 37, 43))
        .border(
            border::rounded(6)
                .width(1)
                .color(Color::from_rgb8(58, 63, 73)),
        )
}

fn subtle_panel_style(_: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style::default()
        .background(Color::from_rgb8(28, 31, 36))
        .border(
            border::rounded(6)
                .width(1)
                .color(Color::from_rgb8(48, 53, 61)),
        )
}

fn color_box_style(color: Rgb, radius: u32) -> iced::widget::container::Style {
    let fill = Color::from_rgb8(color.red, color.green, color.blue);

    iced::widget::container::Style::default()
        .background(fill)
        .border(
            border::rounded(radius)
                .width(1)
                .color(Color::from_rgb8(8, 10, 12)),
        )
}

fn load_saved_effect() -> std::result::Result<Option<Effect>, String> {
    let path = config::user_config_path(Product::G213).map_err(|error| error.to_string())?;
    let saved_config = match config::read_config(path) {
        Ok(config) => config,
        Err(G213Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(error) => return Err(error.to_string()),
    };

    let spec = crate::product::spec_for(saved_config.product);
    effect_from_commands(spec, &saved_config.commands).map_err(|error| error.to_string())
}
