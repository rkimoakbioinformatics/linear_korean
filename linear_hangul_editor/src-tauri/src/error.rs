pub enum Error {
    Config(ConfigError),
    Glyph(GlyphError),
    Font(FontError),
    Kerning(KerningError),
}

pub struct ConfigError {
    pub msg: String,
}

pub struct GlyphError {
    pub msg: String,
}

pub struct FontError {
    pub msg: String,
}

pub struct KerningError {
    pub msg: String,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Error::Config(ConfigError { msg }) => serializer.serialize_str(msg),
            Error::Glyph(GlyphError { msg }) => serializer.serialize_str(msg),
            Error::Font(FontError { msg }) => serializer.serialize_str(msg),
            Error::Kerning(KerningError { msg }) => serializer.serialize_str(msg),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Config(ConfigError { msg }) => write!(f, "{}", msg),
            Error::Glyph(GlyphError { msg }) => write!(f, "{}", msg),
            Error::Font(FontError { msg }) => write!(f, "{}", msg),
            Error::Kerning(KerningError { msg }) => write!(f, "{}", msg),
        }
    }
}
