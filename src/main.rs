use glob::glob;
use human_panic::setup_panic;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg, ArgMatches};
use failure::Error;

use badm::{commands, paths};
use badm::{Config, DirScanner};

fn setup_logging() {
    use env_logger::Builder;

    let mut builder = Builder::new();

    builder
        .filter_module("badm", log::LevelFilter::Trace)
        .format_timestamp(None)
        .init();
}

fn main() -> Result<(), Error> {
    setup_logging();
    setup_panic!();

    let set_dir_subcommand = App::new("set-dir")
        .about("set path of dotfiles directory")
        .version("1.0")
        .display_order(1)
        .arg(
            Arg::with_name("directory")
                .help("directory to store dotfiles")
                .required(true),
        );

    let stow_subcommand = App::new("stow")
        .about(
            "store input files in the dotfiles directory, and replace the file's \
             original path with a symlink",
        )
        .version("0.1")
        .display_order(2)
        .arg(
            Arg::with_name("files")
                .help("path of the file/files to be stored in the dotfiles directory")
                .required(true)
                .multiple(true),
        );

    let deploy_subcommand = App::new("deploy")
        .about(
            "for new configurations, create symlinks in directories relative to the \
             dotfile's directory hierarchy. Directories to replicate the stored \
             dotfile's directory structure will be created if not found.",
        )
        .version("0.1")
        .display_order(3)
        .arg(
            Arg::with_name("dotfiles")
                .help("stored dotfile/s to be deployed to system")
                .required(false)
                .multiple(true),
        )
        .arg(
            Arg::with_name("all")
                .help("deploy all stored dotfiles")
                .long("all")
                .required(false)
                .conflicts_with("dotfiles"),
        );

    let restore_subcommand = App::new("restore")
        .about("restore all dotfiles to their original locations")
        .version("0.1")
        .display_order(4)
        .arg(
            Arg::with_name("dotfiles")
                .help("the dotfiles to restore to original locations")
                .multiple(true)
                .required(true),
        );

    let matches = App::new("badm")
        .about(crate_description!())
        .version(crate_version!())
        .after_help("https://github.com/jakeschurch/badm")
        .author(crate_authors!())
        .subcommands(vec![
            set_dir_subcommand,
            stow_subcommand,
            deploy_subcommand,
            restore_subcommand,
        ])
        .get_matches();

    match matches.subcommand() {
        ("set-dir", Some(set_dir_matches)) => {
            let dir_path = set_dir_matches.value_of("directory").unwrap();
            set_dir(dir_path.into())?
        }
        ("stow", Some(stow_matches)) => stow(stow_matches)?,
        ("deploy", Some(deploy_matches)) => deploy(deploy_matches)?,
        ("restore", Some(restore_matches)) => restore(restore_matches)?,

        // print --help by default
        _ => {
            let output = Command::new("badm")
                .arg("--help")
                .stdout(Stdio::inherit())
                .output()
                .expect("not able to display help message!");

            io::stdout().write_all(&output.stdout).unwrap();
        }
    }
    Ok(())
}

fn validate_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter(|path| path.is_file() && !paths::is_symlink(path))
        .map(|path| {
            if path.is_relative() {
                fs::canonicalize(path)
            } else {
                Ok(path)
            }
        })
        .filter_map(Result::ok)
        .collect::<Vec<PathBuf>>()
}

fn set_dir(path: PathBuf) -> Result<(), Error> {
    match Config::set_dots_dir(&path) {
        Ok(set_path) => {
            info! {"BADM dotfiles directory has been set to: {:?}", set_path};
            Ok(())
        }
        Err(err) => {
            error! {"Could not set BADM dotfiles directory to {:?}. Error: {}", path, err};
            Err(err.into())
        }
    }
}

fn stow(values: &ArgMatches) -> io::Result<()> {
    let mut input_paths = vec![];

    for path in values.values_of("files").unwrap() {
        let paths: Vec<PathBuf> = glob(path).unwrap().filter_map(Result::ok).collect();
        let mut path_vec = validate_paths(paths);

        input_paths.append(&mut path_vec);
    }

    'path: for path in input_paths.into_iter() {
        let dst_path = match commands::store_dotfile(&path) {
            Ok(dst_path) => dst_path,
            Err(err) => {
                error!("Could not stow file {:?}. Error: {}", &path, err);
                continue 'path;
            }
        };

        commands::deploy_dotfile(&dst_path, &path).map_or_else(
            |err| {
                error!(
                    "Stowed file {:?} could not be deployed. Error: {}",
                    path, err
                )
            },
            |_| debug!("{:?} deployed -> {:?}", dst_path, path),
        );
    }

    Ok(())
}

fn deploy(values: &ArgMatches) -> io::Result<()> {
    let dotfiles_dir = Config::get_dots_dir().unwrap();

    let dotfiles = if values.is_present("all") {
        DirScanner::default()
            .recursive()
            .get_entries(&dotfiles_dir)?
    } else {
        let paths: Vec<PathBuf> = values
            .values_of("dotfiles")
            .unwrap()
            .map(PathBuf::from)
            .collect();

        validate_paths(paths)
    };

    for dotfile in dotfiles.into_iter() {
        let dst_path = PathBuf::from("/").join(
            dotfile
                .strip_prefix(&dotfiles_dir)
                .expect("could not strip dotfile path"),
        );

        commands::deploy_dotfile(&dotfile, &dst_path).map_or_else(
            |err| error!("{:?} could not be deployed. Error: {}", dotfile, err),
            |_| debug!("{:?} -> deployed {:?}", dotfile, dst_path),
        );
    }

    Ok(())
}

fn restore(matches: &ArgMatches) -> io::Result<()> {
    let dotfiles: Vec<PathBuf> = matches
        .values_of("dotfiles")
        .unwrap()
        .map(PathBuf::from)
        .collect();

    for dotfile in dotfiles.iter() {
        let _ = commands::restore_dotfile(dotfile.to_path_buf())
            .map_err(|err| error!("{:?} could not be restored. Error: {}", dotfile, err));
    }
    Ok(())
}
