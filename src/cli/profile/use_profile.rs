use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ProfileUseArgs {
    #[facet(args::positional)]
    pub name: String,
}

impl ProfileUseArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        app_state::ensure_initialized(context.app_home())?;
        app_state::set_active_profile(context.app_home(), &self.name)?;
        println!("Now using {}.", self.name);
        Ok(())
    }
}

impl ToArgs for ProfileUseArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
