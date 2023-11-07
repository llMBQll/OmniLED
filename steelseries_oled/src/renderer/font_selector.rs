use serde::de::Error;
use serde::{de, Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FontSelector {
    Default,
    Filesystem(FilesystemSelector),
    System(SystemSelector),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilesystemSelector {
    pub path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_index: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemSelector {
    pub names: Vec<FamilyName>,

    #[serde(default)]
    pub style: Style,

    #[serde(default)]
    pub weight: Weight,

    #[serde(default)]
    pub stretch: Stretch,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FamilyName(
    #[serde(deserialize_with = "deserialize_family_name")] pub font_kit::family_name::FamilyName,
);

fn deserialize_family_name<'de, D>(
    deserializer: D,
) -> Result<font_kit::family_name::FamilyName, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    Ok(match s.as_str() {
        "Serif" => font_kit::family_name::FamilyName::Serif,
        "SansSerif" => font_kit::family_name::FamilyName::SansSerif,
        "Monospace" => font_kit::family_name::FamilyName::Monospace,
        "Cursive" => font_kit::family_name::FamilyName::Cursive,
        "Fantasy" => font_kit::family_name::FamilyName::Fantasy,
        title => font_kit::family_name::FamilyName::Title(title.to_string()),
    })
}

impl Serialize for FamilyName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            font_kit::family_name::FamilyName::Title(title) => serializer.serialize_str(title),
            font_kit::family_name::FamilyName::Serif => serializer.serialize_str("Serif"),
            font_kit::family_name::FamilyName::SansSerif => serializer.serialize_str("SansSerif"),
            font_kit::family_name::FamilyName::Monospace => serializer.serialize_str("Monospace"),
            font_kit::family_name::FamilyName::Cursive => serializer.serialize_str("Cursive"),
            font_kit::family_name::FamilyName::Fantasy => serializer.serialize_str("Fantasy"),
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Style(#[serde(deserialize_with = "deserialize_style")] pub font_kit::properties::Style);

fn deserialize_style<'de, D>(deserializer: D) -> Result<font_kit::properties::Style, D::Error>
where
    D: de::Deserializer<'de>,
{
    const NAMES: &[&str] = &["Normal", "Italic", "Oblique"];

    let s: String = de::Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "Normal" => Ok(font_kit::properties::Style::Normal),
        "Italic" => Ok(font_kit::properties::Style::Italic),
        "Oblique" => Ok(font_kit::properties::Style::Oblique),
        value => Err(Error::unknown_variant(value, NAMES)),
    }
}

impl Serialize for Style {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self.0 {
            font_kit::properties::Style::Normal => "Normal",
            font_kit::properties::Style::Italic => "Italic",
            font_kit::properties::Style::Oblique => "Oblique",
        };
        serializer.serialize_str(s)
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Weight(
    #[serde(deserialize_with = "deserialize_weight")] pub font_kit::properties::Weight,
);

fn deserialize_weight<'de, D>(deserializer: D) -> Result<font_kit::properties::Weight, D::Error>
where
    D: de::Deserializer<'de>,
{
    const NAMES: &[&str] = &[
        "Thin",
        "ExtraLight",
        "Light",
        "Normal",
        "Medium",
        "SemiBold",
        "Bold",
        "ExtraBold",
        "Black",
    ];

    let s: String = de::Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "Thin" => Ok(font_kit::properties::Weight::THIN),
        "ExtraLight" => Ok(font_kit::properties::Weight::EXTRA_LIGHT),
        "Light" => Ok(font_kit::properties::Weight::LIGHT),
        "Normal" => Ok(font_kit::properties::Weight::NORMAL),
        "Medium" => Ok(font_kit::properties::Weight::MEDIUM),
        "SemiBold" => Ok(font_kit::properties::Weight::SEMIBOLD),
        "Bold" => Ok(font_kit::properties::Weight::BOLD),
        "ExtraBold" => Ok(font_kit::properties::Weight::EXTRA_BOLD),
        "Black" => Ok(font_kit::properties::Weight::BLACK),
        value => Err(Error::unknown_variant(value, NAMES)),
    }
}

impl Serialize for Weight {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let weight = self.0;
        let s = {
            if weight.0 == font_kit::properties::Weight::THIN.0 {
                "Thin"
            } else if weight.0 == font_kit::properties::Weight::EXTRA_LIGHT.0 {
                "Extra_light"
            } else if weight.0 == font_kit::properties::Weight::LIGHT.0 {
                "Light"
            } else if weight.0 == font_kit::properties::Weight::NORMAL.0 {
                "Normal"
            } else if weight.0 == font_kit::properties::Weight::MEDIUM.0 {
                "Medium"
            } else if weight.0 == font_kit::properties::Weight::SEMIBOLD.0 {
                "Semi_bold"
            } else if weight.0 == font_kit::properties::Weight::BOLD.0 {
                "Bold"
            } else if weight.0 == font_kit::properties::Weight::EXTRA_BOLD.0 {
                "Extra_bold"
            } else if weight.0 == font_kit::properties::Weight::BLACK.0 {
                "Black"
            } else {
                unreachable!();
            }
        };
        serializer.serialize_str(s)
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Stretch(
    #[serde(deserialize_with = "deserialize_stretch")] pub font_kit::properties::Stretch,
);

fn deserialize_stretch<'de, D>(deserializer: D) -> Result<font_kit::properties::Stretch, D::Error>
where
    D: de::Deserializer<'de>,
{
    const NAMES: &[&str] = &[
        "UltraCondensed",
        "ExtraCondensed",
        "Condensed",
        "SemiCondensed",
        "Normal",
        "SemiExpanded",
        "Expanded",
        "ExtraExpanded",
        "UltraExpanded",
    ];

    let s: String = de::Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "UltraCondensed" => Ok(font_kit::properties::Stretch::ULTRA_CONDENSED),
        "ExtraCondensed" => Ok(font_kit::properties::Stretch::EXTRA_CONDENSED),
        "Condensed" => Ok(font_kit::properties::Stretch::CONDENSED),
        "SemiCondensed" => Ok(font_kit::properties::Stretch::SEMI_CONDENSED),
        "Normal" => Ok(font_kit::properties::Stretch::NORMAL),
        "SemiExpanded" => Ok(font_kit::properties::Stretch::SEMI_EXPANDED),
        "Expanded" => Ok(font_kit::properties::Stretch::EXPANDED),
        "ExtraExpanded" => Ok(font_kit::properties::Stretch::EXTRA_EXPANDED),
        "UltraExpanded" => Ok(font_kit::properties::Stretch::ULTRA_EXPANDED),
        value => Err(Error::unknown_variant(value, NAMES)),
    }
}

impl Serialize for Stretch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let stretch = self.0;
        let s = {
            if stretch.0 == font_kit::properties::Stretch::ULTRA_CONDENSED.0 {
                "UltraCondensed"
            } else if stretch.0 == font_kit::properties::Stretch::EXTRA_CONDENSED.0 {
                "ExtraCondensed"
            } else if stretch.0 == font_kit::properties::Stretch::CONDENSED.0 {
                "Condensed"
            } else if stretch.0 == font_kit::properties::Stretch::SEMI_CONDENSED.0 {
                "SemiCondensed"
            } else if stretch.0 == font_kit::properties::Stretch::NORMAL.0 {
                "Normal"
            } else if stretch.0 == font_kit::properties::Stretch::SEMI_EXPANDED.0 {
                "SemiExpanded"
            } else if stretch.0 == font_kit::properties::Stretch::EXPANDED.0 {
                "Expanded"
            } else if stretch.0 == font_kit::properties::Stretch::EXTRA_EXPANDED.0 {
                "ExtraExpanded"
            } else if stretch.0 == font_kit::properties::Stretch::ULTRA_EXPANDED.0 {
                "UltraExpanded"
            } else {
                unreachable!()
            }
        };
        serializer.serialize_str(s)
    }
}
