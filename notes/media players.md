## Status (2026-02-22)

Implemented in `vetchricore`:

- [x] `media player list`
- [x] `media player add <key> <path>` (path canonicalized before persist)
- [x] `media player show <key>`
- [x] `media player default set <key>`
- [x] `media player default show`
- [x] `media player detect now`
- [x] `--output-format auto|text|json` for `list` and `detect now`
- [x] PATH-based discovery of well-known `.exe` names from syncplay-style player set
- [x] unsupported-player labeling support (e.g. `wmplayer` is marked unsupported)

Remaining design work:

- [ ] global output format as a top-level flag across all commands
- [ ] richer output targets (`facet-pretty`, `tsv`, etc.)
- [ ] common output trait so command handlers can render consistently

---

We want to track the media players that the user has on their device.

We should list all media players we can discover.

We may know of media players that are not compatible with our software; for those that we find we should include them and explicitly mention they are not supported.

This provides users with the most information.

The CLI should be able to like


```pwsh
vetchricore.exe media player list # configured players only
vetchricore.exe media player add vlc D:\programs\vlc.exe # canonicalize the path before persisting
vetchricore.exe media player new vlc D:\programs\vlc.exe
vetchricore.exe media player set vlc D:\programs\vlc.exe
vetchricore.exe media player update vlc D:\programs\vlc.exe
vetchricore.exe media player create vlc D:\programs\vlc.exe
vetchricore.exe media player show vlc # show path and any other info if any
vetchricore.exe media player default set vlc
vetchricore.exe media player default show
vetchricore.exe media player detect now --walk ask --walk-timeout 25s
vetchricore.exe media player discover now --walk ask --walk-timeout 25s
vetchricore.exe media player detect now --walk true --walk-timeout 25s
vetchricore.exe media player detect now --walk true --walk-timeout 25s --walk-roots "C:\\Program Files;D:\\Apps"
vetchricore.exe media player detect now --walk false
vetchricore.exe media player list # should have a `--output-format json` behaviour
VLC (supported) D:\programs\vlc.exe # colourized output
```


here is an example of an existing thing I did for output format

```rust
use crate::audio::list_audio_input_devices;
use crate::cli::to_args::ToArgs;
use arbitrary::Arbitrary;
use clap::Args;
use clap::ValueEnum;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::colors::BrightBlack;
use color_eyre::owo_colors::colors::Yellow;
use eyre::Result;
use facet::Facet;
use facet_pretty::ColorMode;
use facet_pretty::PrettyPrinter;
use std::ffi::OsString;
use std::io::IsTerminal;
use std::ops::Deref;

/// List microphones.
#[derive(Args, Debug, Arbitrary, PartialEq)]
pub struct MicListArgs {
    /// Output format.
    #[clap(long, value_enum, default_value_t = OutputFormat::Auto)]
    pub output_format: OutputFormat,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq, Hash, Arbitrary)]
pub enum OutputFormat {
    Auto,
    Text,
    Facet,
    Json,
}
impl MicListArgs {
    pub fn invoke(mut self) -> Result<()> {
        let is_terminal = std::io::stdout().is_terminal();
        if matches!(self.output_format, OutputFormat::Auto) {
            self.output_format = if is_terminal {
                OutputFormat::Text
            } else {
                OutputFormat::Json
            };
        }

        let devices = list_audio_input_devices()?;

        match self.output_format {
            OutputFormat::Auto => unreachable!(),
            OutputFormat::Text => {
                if devices.is_empty() {
                    println!("{}", "No microphones found.".red());
                    return Ok(());
                }

                for device in devices {
                    let default_marker = if device.is_default { " (default)" } else { "" };
                    println!(
                        "({id}) {name} {default_marker}",
                        id = device.id.deref().fg::<BrightBlack>(),
                        name = device.name,
                        default_marker = default_marker.fg::<Yellow>()
                    );
                }
            }
            OutputFormat::Json | OutputFormat::Facet => {
                // emit json
                structstruck::strike! {
                    #[structstruck::each[derive(Facet)]]
                    struct MicListOutput {
                        microphones: Vec<struct Mic {
                            id: String,
                            name: String,
                            is_default: bool,
                        }>,
                    }
                }
                let mics: Vec<Mic> = devices
                    .into_iter()
                    .map(|device| Mic {
                        id: device.id.0,
                        name: device.name,
                        is_default: device.is_default,
                    })
                    .collect();
                match (is_terminal, &self.output_format) {
                    (true, OutputFormat::Facet) => {
                        let output = MicListOutput { microphones: mics };
                        let out = PrettyPrinter::new()
                            .with_colors(ColorMode::Always)
                            .with_doc_comments(true)
                            .format(&output);
                        println!("{}", out);
                    }
                    (false, OutputFormat::Facet) => {
                        let output = MicListOutput { microphones: mics };
                        let out = PrettyPrinter::new()
                            .with_colors(ColorMode::Never)
                            .format(&output);
                        println!("{}", out);
                    }
                    (true, OutputFormat::Json) => {
                        // Output array directly for easier PowerShell piping
                        let json = facet_json::to_string_pretty(&mics)?;
                        println!("{}", json);
                    }
                    (false, OutputFormat::Json) => {
                        // Output array directly for easier PowerShell piping
                        let json = facet_json::to_string(&mics)?;
                        println!("{}", json);
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }
}

impl ToArgs for MicListArgs {
    fn to_args(&self) -> Vec<OsString> {
        Vec::new()
    }
}

```

from teamy-rust-windows-utils\src\cli\command\mic\list\mic_list_cli.rs


it would be good if we could make output format a global flag.

to support that, each invoke fn would have to return something that we know how to pump and process.


Display + Facet + PrettyOutput (new trait, takes a writer and writes the coloured ascii to it)

enum OutputFormat {
    Auto, # textpretty if is a terminal, json otherwise (pipe scenario)
    Json, # facet-json
    Tsv, # facet-tsv maybe exists
    FacetPretty, # facet-pretty exists
    TextPretty # manual print logic for each output
    Text, # print without ascii colouration, ideally without duplicating too much logic
}


All of our commands should support each output format.



syncplay has a list of supported players here if that helps us determine where to look for media players
../../syncplay/syncplay/players
../../syncplay/syncplay/players/__init__.py
../../syncplay/syncplay/players/basePlayer.py
../../syncplay/syncplay/players/iina.py
../../syncplay/syncplay/players/ipc_iina.py
../../syncplay/syncplay/players/memento.py
../../syncplay/syncplay/players/mpc.py
../../syncplay/syncplay/players/mpcbe.py
../../syncplay/syncplay/players/mplayer.py
../../syncplay/syncplay/players/mpv.py
../../syncplay/syncplay/players/mpvnet.py
../../syncplay/syncplay/players/playerFactory.py
../../syncplay/syncplay/players/vlc.py

our discovery logic should be able to look through $env:PATH to check for well-known .exe files that match the players our code can detect.