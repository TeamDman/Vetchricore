use arbitrary::Arbitrary;
use eyre::bail;
use facet::Facet;
use std::fmt;
use std::io::IsTerminal;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Arbitrary)]
#[repr(u8)]
pub enum OutputFormat {
    Text,
    Json,
    PrettyJson,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => f.write_str("text"),
            Self::Json => f.write_str("json"),
            Self::PrettyJson => f.write_str("pretty-json"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "pretty-json" | "pretty_json" | "prettyjson" => Ok(Self::PrettyJson),
            _ => bail!(
                "Unsupported output format '{}'. Use text, json, or pretty-json.",
                value
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Arbitrary)]
#[repr(u8)]
pub enum OutputFormatArg {
    Auto,
    Text,
    Json,
    PrettyJson,
}

impl fmt::Display for OutputFormatArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => f.write_str("auto"),
            Self::Text => f.write_str("text"),
            Self::Json => f.write_str("json"),
            Self::PrettyJson => f.write_str("pretty-json"),
        }
    }
}

impl OutputFormatArg {
    #[must_use]
    pub fn as_cli_token(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Text => "Text",
            Self::Json => "Json",
            Self::PrettyJson => "PrettyJson",
        }
    }

    #[must_use]
    pub fn resolve(self) -> OutputFormat {
        match self {
            Self::Auto => {
                if std::io::stdout().is_terminal() {
                    OutputFormat::Text
                } else {
                    OutputFormat::PrettyJson
                }
            }
            Self::Text => OutputFormat::Text,
            Self::Json => OutputFormat::Json,
            Self::PrettyJson => OutputFormat::PrettyJson,
        }
    }
}

impl FromStr for OutputFormatArg {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "pretty-json" | "pretty_json" | "prettyjson" => Ok(Self::PrettyJson),
            _ => bail!(
                "Unsupported output format '{}'. Use auto, text, json, or pretty-json.",
                value
            ),
        }
    }
}
