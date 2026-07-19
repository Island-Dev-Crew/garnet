//! `garnet mcp-serve --package <dir>` - one sealed local Tier 1 MCP host.

use crate::minimum_shelf::MinimumShelfPackage;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        println!("usage: garnet mcp-serve --package <sealed-package-dir>");
        return ExitCode::SUCCESS;
    }
    let [flag, path] = args else {
        eprintln!("usage: garnet mcp-serve --package <sealed-package-dir>");
        return ExitCode::from(2);
    };
    if flag != "--package" {
        eprintln!("usage: garnet mcp-serve --package <sealed-package-dir>");
        return ExitCode::from(2);
    }

    let package = match MinimumShelfPackage::load(&PathBuf::from(path)) {
        Ok(package) => package,
        Err(error) => {
            eprintln!("garnet mcp-serve: {error}");
            return ExitCode::from(1);
        }
    };
    let mut host = package.into_host();
    if let Err(error) = crate::mcp_stdio::set_binary_stdio() {
        eprintln!("garnet mcp-serve: could not set raw-byte stdio: {error}");
        return ExitCode::from(1);
    }

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    match crate::mcp_stdio::serve(&mut host, &mut input, &mut output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("garnet mcp-serve: stdio session failed: {error}");
            ExitCode::from(1)
        }
    }
}
