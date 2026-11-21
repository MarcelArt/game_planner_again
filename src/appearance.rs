use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Colors {
    pub background: u32,
    pub foreground: u32,
    pub accent: u32,
}

impl Colors {
    pub const fn new(background: u32, foreground: u32, accent: u32) -> Self {
        Self {
            background,
            foreground,
            accent,
        }
    }
}

macro_rules! presets {
    ($($id:ident, $name:literal, $dark:expr, $light:expr;)*) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Preset { #[default] Default, $($id,)* Custom }

        impl Preset {
            pub const ALL: &[Self] = &[Self::Default, $(Self::$id,)* Self::Custom];
            pub fn name(self) -> &'static str {
                match self { Self::Default => "Default", $(Self::$id => $name,)* Self::Custom => "Custom" }
            }
            fn variants(self) -> [Colors; 2] {
                match self {
                    $(Self::$id => [$dark, $light],)*
                    Self::Default | Self::Custom => [
                        Colors::new(0x191a1c, 0xffffff, 0xfbd960),
                        Colors::new(0xe7e5df, 0x141410, 0x087486),
                    ],
                }
            }
        }
    };
}

presets! {
    Catppuccin, "Catppuccin", Colors::new(0x1e1e2e, 0xcdd6f4, 0xcba6f7), Colors::new(0xeff1f5, 0x4c4f69, 0x8839ef);
    Dracula, "Dracula", Colors::new(0x282a36, 0xf8f8f2, 0xbd93f9), Colors::new(0xf8f8f2, 0x282a36, 0x7645a8);
    Gruvbox, "Gruvbox", Colors::new(0x282828, 0xebdbb2, 0xfabd2f), Colors::new(0xfbf1c7, 0x3c3836, 0x9d5000);
    Monokai, "Monokai", Colors::new(0x272822, 0xf8f8f2, 0xa6e22e), Colors::new(0xfaf9f5, 0x383830, 0x567700);
    RosePine, "Rosé Pine", Colors::new(0x191724, 0xe0def4, 0xebbcba), Colors::new(0xfaf4ed, 0x575279, 0x907aa9);
    TokyoNight, "Tokyo Night", Colors::new(0x1a1b26, 0xc0caf5, 0x7aa2f7), Colors::new(0xe1e2e7, 0x3760bf, 0x2e5dac);
    Ayu, "Ayu", Colors::new(0x0b0e14, 0xbfbdb6, 0xe6b450), Colors::new(0xf8f9fa, 0x575f66, 0xa76000);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub light: bool,
    pub preset: Preset,
    pub custom: [Option<Colors>; 2],
}

impl Appearance {
    pub fn colors(self) -> Colors {
        let i = usize::from(self.light);
        if self.preset == Preset::Custom
            && let Some(colors) = self.custom[i]
        {
            return colors;
        }
        self.preset.variants()[i]
    }
}
