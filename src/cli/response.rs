use crate::cli::output_format::OutputFormat;
use eyre::Result;
use facet::Facet;
use std::fmt::Display;

#[derive(Default)]
pub struct CliResponse {
    renderer: Option<Box<dyn DeferredRender>>,
}

impl std::fmt::Debug for CliResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.renderer {
            None => f.write_str("CliResponse::Empty"),
            Some(_) => f.write_str("CliResponse::Value(..)"),
        }
    }
}

trait DeferredRender {
    fn render(&self, output_format: OutputFormat) -> Result<String>;
}

struct FacetDeferredRender<T> {
    value: T,
}

impl<T> DeferredRender for FacetDeferredRender<T>
where
    T: for<'a> Facet<'a> + Display,
{
    fn render(&self, output_format: OutputFormat) -> Result<String> {
        match output_format {
            OutputFormat::Text => Ok(self.value.to_string()),
            OutputFormat::Json => Ok(facet_json::to_string(&self.value)?),
            OutputFormat::PrettyJson => Ok(facet_json::to_string_pretty(&self.value)?),
        }
    }
}

impl CliResponse {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn write(self, output_format: OutputFormat) -> Result<()> {
        if let Some(renderer) = self.renderer {
            let body = renderer.render(output_format)?;
            println!("{body}");
        }
        Ok(())
    }
}

impl<T> From<T> for CliResponse
where
    T: for<'a> Facet<'a> + Display + 'static,
{
    fn from(value: T) -> Self {
        Self {
            renderer: Some(Box::new(FacetDeferredRender { value })),
        }
    }
}
