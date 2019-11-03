// TODO: create integration tests for main

#![allow(clippy::all)]
// TEMP: since in large dev production
#![allow(dead_code)]

use std::env;
use std::io;
use std::path::{Path, PathBuf};

#[macro_use]
extern crate clap;

use clap::{App, Arg, ArgMatches, Values};

use badm_core::commands::{deploy_dotfile, restore_dotfile, store_dotfile};
use badm_core::paths::{is_symlink, sanitize_path};
use badm_core::{Config, DirectoryScanner};

fn main() -> io::Result<()> {
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
                .multiple(true),
        )
        .arg(
            Arg::with_name("all")
                .help("deploy all stored dotfiles")
                .long("all")
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
        .author(crate_authors!())
        .after_help("https://github.com/jakeschurch/badm")
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
            set_dir(dir_path)?
        }
        ("stow", Some(stow_matches)) => stow(stow_matches)?,
        ("deploy", Some(deploy_matches)) => deploy(deploy_matches)?,
        ("restore", Some(restore_matches)) => restore(restore_matches)?,
        _ => unreachable!(),
    }
    Ok(())
}

fn set_dir<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let path = path.as_ref().to_path_buf();

    let set_path = Config::set_dots_dir(path)?;

    println! {"BADM dotfiles path has been set to: {:?}", set_path};
    Ok(())
}

fn stow(values: &ArgMatches) -> io::Result<()> {
    let mut input_paths = vec![];

    // TODO: push up to own function
    // prepare paths
    for path in values.values_of("files").unwrap() {
        let paths = glob(path)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|path| path.is_file())
            .collect::<Vec<PathBuf>>();
        input_paths.push(paths);
    }

    for path in input_paths.into_iter().flatten() {
        let src_path = sanitize_path(&path)?;

        // TODO: push down is symlink and return error
        if src_path.is_file() && !is_symlink(&src_path)? {
            let dst_path = store_dotfile(&src_path)?;
            deploy_dotfile(&dst_path, &src_path)?;
        };
    }
    Ok(())
}

fn deploy(matches: &ArgMatches) -> io::Result<()> {
    let dotfiles_dir = Config::get_dots_dir().unwrap();

    let dotfiles = if matches.is_present("all") {
        DirectoryScanner::new().get_entries(&dotfiles_dir)?
    } else {
        matches
            .values_of("dotfiles")
            .unwrap()
            .into_iter()
            .map(|path| PathBuf::from(path))
            .collect::<Vec<PathBuf>>()
    };

    for dotfile in dotfiles.into_iter() {
        deploy_dotfile(&dotfile, &dotfiles_dir)?;
    }

    Ok(())
}

fn restore(matches: &ArgMatches) -> io::Result<()> {
    let dotfiles = matches.values_of("dotfiles").unwrap();
    for dotfile in dotfiles.into_iter() {
        let path = PathBuf::from(dotfile);
        restore_dotfile(path)?;
    }
    Ok(())
}
