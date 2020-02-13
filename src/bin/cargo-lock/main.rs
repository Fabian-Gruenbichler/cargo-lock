//! The `cargo lock` subcommand

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, unused_qualifications)]

use cargo_lock::{package, Dependency, Lockfile, ResolveVersion};
#[cfg(feature = "cli")]
use gumdrop::Options;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::exit,
};

#[cfg(feature = "dependency-tree")]
use cargo_lock::dependency::graph::EdgeDirection;

/// Wrapper toplevel command for the `cargo lock` subcommand
#[derive(Options)]
#[cfg(feature = "cli")]
enum CargoLock {
    #[options(help = "the `cargo lock` Cargo subcommand")]
    Lock(Command),
}

/// `cargo lock` subcommands
#[derive(Debug, Options)]
#[cfg(feature = "cli")]
enum Command {
    /// The `cargo lock list` subcommand
    #[options(help = "list packages in Cargo.toml")]
    List(ListCmd),

    /// The `cargo lock translate` subcommand
    #[options(help = "translate a Cargo.toml file")]
    Translate(TranslateCmd),

    /// The `cargo lock tree` subcommand
    #[cfg(feature = "dependency-tree")]
    #[options(help = "print a dependency tree for the given dependency")]
    Tree(TreeCmd),
}

/// The `cargo lock list` subcommand
#[derive(Debug, Options)]
#[cfg(feature = "cli")]
struct ListCmd {
    /// Input `Cargo.lock` file
    #[options(short = "f", help = "input Cargo.lock file to translate")]
    file: Option<PathBuf>,
}

#[cfg(feature = "cli")]
impl ListCmd {
    /// Display dependency summary from `Cargo.lock`
    pub fn run(&self) {
        for package in &load_lockfile(&self.file).packages {
            println!("- {}", Dependency::from(package));
        }
    }
}

/// The `cargo lock translate` subcommand
#[derive(Debug, Options)]
#[cfg(feature = "cli")]
struct TranslateCmd {
    /// Input `Cargo.lock` file
    #[options(short = "f", help = "input Cargo.lock file to translate")]
    file: Option<PathBuf>,

    /// Output `Cargo.lock` file
    #[options(short = "o", help = "output Cargo.lock file (default STDOUT)")]
    output: Option<PathBuf>,

    /// Cargo.lock format version to translate to
    #[options(short = "v", help = "Cargo.lock resolve version to output")]
    version: Option<ResolveVersion>,
}

#[cfg(feature = "cli")]
impl TranslateCmd {
    /// Translate `Cargo.lock` to a different format version
    pub fn run(&self) {
        let output = self
            .output
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or_else(|| Path::new("-"));

        let mut lockfile = load_lockfile(&self.file);

        lockfile.version = self.version.unwrap_or_default();
        let lockfile_toml = lockfile.to_string();

        if output == Path::new("-") {
            println!("{}", &lockfile_toml);
        } else {
            fs::write(output, lockfile_toml.as_bytes()).unwrap_or_else(|e| {
                eprintln!("*** error: {}", e);
                exit(1);
            });
        }
    }
}

/// The `cargo lock tree` subcommand
#[cfg(feature = "cli")]
#[cfg(feature = "dependency-tree")]
#[derive(Debug, Options)]
struct TreeCmd {
    /// Input `Cargo.lock` file
    #[options(short = "f", help = "input Cargo.lock file to translate")]
    file: Option<PathBuf>,

    /// Dependencies names to draw a tree for
    #[options(free, help = "dependency names to draw trees for")]
    dependencies: Vec<package::Name>,
}

#[cfg(feature = "cli")]
#[cfg(feature = "dependency-tree")]
impl TreeCmd {
    /// Display dependency trees from `Cargo.lock`
    pub fn run(&self) {
        let lockfile = load_lockfile(&self.file);

        let tree = lockfile.dependency_tree().unwrap_or_else(|e| {
            eprintln!("*** error: {}", e);
            exit(1);
        });

        // TODO(tarcieri): detect root package(s), automatically use those?
        if self.dependencies.is_empty() {
            eprintln!("*** error: no dependency names given");
            exit(1);
        }

        for (i, dep) in self.dependencies.iter().enumerate() {
            if i > 0 {
                println!();
            }

            let package = lockfile
                .packages
                .iter()
                .find(|pkg| pkg.name == *dep)
                .unwrap_or_else(|| {
                    eprintln!("*** error: invalid dependency name: `{}`", dep);
                    exit(1);
                });

            let index = tree.nodes()[&package.into()];
            tree.render(&mut io::stdout(), index, EdgeDirection::Incoming)
                .unwrap();
        }
    }
}

/// Load a lockfile from the given path (or `Cargo.toml`)
fn load_lockfile(path: &Option<PathBuf>) -> Lockfile {
    let path = path
        .as_ref()
        .map(AsRef::as_ref)
        .unwrap_or_else(|| Path::new("Cargo.lock"));

    Lockfile::load(path).unwrap_or_else(|e| {
        eprintln!("*** error: {}", e);
        exit(1);
    })
}

#[cfg(feature = "cli")]
fn main() {
    let args = env::args().collect::<Vec<_>>();

    let CargoLock::Lock(cmd) = CargoLock::parse_args_default(&args[1..]).unwrap_or_else(|e| {
        eprintln!("*** error: {}", e);
        eprintln!("USAGE:");
        eprintln!("{}", Command::usage());
        exit(1);
    });

    match cmd {
        Command::List(list) => list.run(),
        Command::Translate(translate) => translate.run(),
        #[cfg(feature = "dependency-tree")]
        Command::Tree(tree) => tree.run(),
    }
}

#[cfg(not(feature = "cli"))]
fn main() {
    // intentionally empty
}
