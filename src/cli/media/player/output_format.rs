use eyre::bail;
use std::io::IsTerminal;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl FromStr for OutputFormat {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => bail!("Unsupported output format '{}'. Use text or json.", value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormatArg {
    Auto,
    Some(OutputFormat),
}

impl OutputFormatArg {
    #[must_use]
    pub fn resolve(self) -> OutputFormat {
        match self {
            Self::Auto => {
                if std::io::stdout().is_terminal() {
                    OutputFormat::Text
                } else {
                    OutputFormat::Json
                }
            }
            Self::Some(format) => format,
        }
    }
}

impl FromStr for OutputFormatArg {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            _ => Ok(Self::Some(value.parse::<OutputFormat>()?)),
        }
    }
}
