//! `garnet-registry-stub` — generate or verify a filesystem-backed Garnet
//! registry index. v0.1 stub: no HTTP, no auth, no publish, no signing.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") => cmd_build(&args[1..]),
        Some("verify") => cmd_verify(&args[1..]),
        Some("--help") | Some("-h") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("garnet-registry-stub: unknown subcommand `{other}`");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    eprintln!("usage: garnet-registry-stub <build|verify> <registry-dir>");
    eprintln!();
    eprintln!("  build   Scan <registry-dir>/<name>/<version>/ packages, hash each");
    eprintln!("          file with BLAKE3, and write <registry-dir>/index.json.");
    eprintln!("  verify  Re-hash every package and check it against index.json.");
    eprintln!();
    eprintln!("  v0.1 stub: filesystem-backed only. No HTTP transport, no auth,");
    eprintln!("  no publish flow, no signature verification.");
}

fn registry_dir(args: &[String]) -> Result<PathBuf, String> {
    match args.first() {
        Some(dir) => Ok(PathBuf::from(dir)),
        None => Err("missing <registry-dir> argument".to_string()),
    }
}

fn cmd_build(args: &[String]) -> ExitCode {
    let dir = match registry_dir(args) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("garnet-registry-stub build: {e}");
            return ExitCode::from(2);
        }
    };
    match garnet_registry_stub::build_index(&dir) {
        Ok(index) => {
            if let Err(e) = garnet_registry_stub::write_index(&dir, &index) {
                eprintln!("garnet-registry-stub build: {e}");
                return ExitCode::from(1);
            }
            let pkgs = index.packages.len();
            let vers: usize = index.packages.values().map(|p| p.versions.len()).sum();
            println!(
                "garnet-registry-stub: wrote {}/index.json ({pkgs} package(s), {vers} version(s))",
                dir.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("garnet-registry-stub build: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_verify(args: &[String]) -> ExitCode {
    let dir = match registry_dir(args) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("garnet-registry-stub verify: {e}");
            return ExitCode::from(2);
        }
    };
    let index = match garnet_registry_stub::load_index(&dir) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("garnet-registry-stub verify: {e}");
            return ExitCode::from(1);
        }
    };
    let mut checked = 0usize;
    for (name, package) in &index.packages {
        for (version, entry) in &package.versions {
            let pkg_dir = match garnet_registry_stub::package_dir(&dir, entry) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("garnet-registry-stub verify: {name}@{version}: {e}");
                    return ExitCode::from(1);
                }
            };
            if let Err(e) = garnet_registry_stub::verify_package(&pkg_dir, entry) {
                eprintln!("garnet-registry-stub verify: {name}@{version}: {e}");
                return ExitCode::from(1);
            }
            checked += 1;
        }
    }
    println!("garnet-registry-stub: verified {checked} package version(s) OK");
    ExitCode::SUCCESS
}
