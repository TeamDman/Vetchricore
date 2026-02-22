param(
    [switch]$Online
)

$offlineFlag = if ($Online) { "" } else { "--offline" }
cargo install --path . --locked $offlineFlag