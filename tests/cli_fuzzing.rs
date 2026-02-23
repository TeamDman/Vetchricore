//! CLI fuzzing tests using arbitrary-generated CLI instances.

use arbitrary::Arbitrary;
use rand::rngs::OsRng;
use rand::TryRngCore;
use vetchricore::cli::Cli;
use vetchricore::cli::ToArgs;

fn parse_cli_from_args(args: &[std::ffi::OsString]) -> Result<Cli, figue::DriverError> {
    let cli_args = args
        .iter()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    figue::Driver::new(
        figue::builder::<Cli>()
            .expect("schema should be valid")
            .cli(|c| c.args(cli_args).strict())
            .build(),
    )
    .run()
    .into_result()
    .map(|output| output.get_silent())
}

#[test]
fn fuzz_cli_args_consistency() {
    // Test that the same CLI instance always produces the same args
    let mut data = vec![123u8; 1024];
    let mut rng = arbitrary::Unstructured::new(&data);

    for i in 0..5000 {
        let cli = match Cli::arbitrary(&mut rng) {
            Ok(cli) => cli,
            Err(_) => {
                data = vec![(i * 2) as u8; 1024];
                rng = arbitrary::Unstructured::new(&data);
                Cli::arbitrary(&mut rng).expect("Failed to generate CLI instance")
            }
        };

        let args1 = cli.to_args();
        let args2 = cli.to_args();

        assert_eq!(
            args1, args2,
            "CLI.to_args() should be deterministic for iteration {i}",
        );
    }
}

#[test]
fn fuzz_cli_args_roundtrip() {
    // Test that to_args output can be parsed back into the same CLI structure
    let mut data = vec![0u8; 1024];
    let mut rng = arbitrary::Unstructured::new(&data);
    let mut os_rng = OsRng;

    for i in 0..1000 {
        let cli = match Cli::arbitrary(&mut rng) {
            Ok(cli) => cli,
            Err(_) => {
                os_rng
                    .try_fill_bytes(&mut data)
                    .expect("Failed to get OS random bytes");
                rng = arbitrary::Unstructured::new(&data);
                Cli::arbitrary(&mut rng).expect("Failed to generate CLI instance")
            }
        };

        let args = cli.to_args();
        let parsed_cli = parse_cli_from_args(&args).unwrap_or_else(|error| {
            panic!(
                "Failed to parse CLI args on iteration {i}: {error:?}\nOriginal CLI: {cli:?}\nArgs: {args:?}"
            )
        });

        assert_eq!(
            cli, parsed_cli,
            "CLI roundtrip failed on iteration {i}: original={cli:?} parsed={parsed_cli:?} args={args:?}"
        );
    }
}

