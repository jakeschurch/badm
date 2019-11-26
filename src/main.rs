use glob::glob;
use human_panic::setup_panic;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[macro_use] extern crate clap;

use clap::{App, Arg, ArgMatches};
use failure::Error;

use badm::commands;
use badm::paths;
use badm::{Config, DirScanner};

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

fn main() -> Result<(), Error> {
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
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("all")
                .help("deploy all stored dotfiles")
                .long("all")
                .conflicts_with("dotfiles")
                .required(true),
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
        },
        ("stow", Some(stow_matches)) => stow(stow_matches)?,
        ("deploy", Some(deploy_matches)) => deploy(deploy_matches)?,
        ("restore", Some(restore_matches)) => restore(restore_matches)?,
        _ => {
            let output = Command::new("badm")
                .arg("--help")
                .stdout(Stdio::inherit())
                .output()
                .expect("not able to display help message!");

            io::stdout().write_all(&output.stdout).unwrap();
        },
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

    for path in values.values_of("files").unwrap() {
        let paths: Vec<PathBuf> = glob(path).unwrap().filter_map(Result::ok).collect();
        let mut path_vec = validate_paths(paths);

        input_paths.append(&mut path_vec);
    }

    for path in input_paths.into_iter() {
        let dst_path = commands::store_dotfile(&path)?;
        commands::deploy_dotfile(&dst_path, &path)?;
    }
    Ok(())
}

fn deploy(values: &ArgMatches) -> io::Result<()> {
    println!("inside of deploy");
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
        println!("{:?}", dotfile);

        let dst_path = PathBuf::from("/").join(
            dotfile
                .strip_prefix(&dotfiles_dir)
                .expect("could not strip dotfile path"),
        );
        println!("dst path: {:?}", dst_path);

        commands::deploy_dotfile(&dotfile, &dst_path)?;
    }

    Ok(())
}

fn restore(matches: &ArgMatches) -> io::Result<()> {
    let dotfiles: Vec<PathBuf> = matches
        .values_of("dotfiles")
        .unwrap()
        .map(PathBuf::from)
        .collect();

    for dotfile in dotfiles.into_iter() {
        commands::restore_dotfile(dotfile)?;
    }
    Ok(())
}
