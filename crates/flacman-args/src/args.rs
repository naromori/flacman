use clap::{Arg, ArgAction, ArgMatches, Command};
use std::process;


pub fn build_cli() -> Command {
    Command::new("flacman")
        .version(env!("CARGO_PKG_VERSION"))
        .author("naromori")
        .about("Pacman-style music package manager")
        .arg(
            Arg::new("sync")
                .short('S')
                .long("sync")
                .help("Download music from remote sources")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["query", "remove", "update"]),
        )
        .arg(
            Arg::new("query")
                .short('Q')
                .long("query")
                .help("Query local music library")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["sync", "remove", "update"]),
        )
        .arg(
            Arg::new("remove")
                .short('R')
                .long("remove")
                .help("Remove music from library")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["sync", "query", "update"]),
        )
        .arg(
            Arg::new("update")
                .short('U')
                .long("update")
                .help("Update/move music files into repository")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["sync", "query", "remove"]),
        )
        .arg(
            Arg::new("artist")
                .short('A')
                .long("artist")
                .help("Target: Artist (download full discography)")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["album", "track"])
                .requires("sync"),
        )
        .arg(
            Arg::new("album")
                .short('a')
                .long("album")
                .help("Target: Album")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["artist", "track"])
                .requires("sync"),
        )
        .arg(
            Arg::new("track")
                .short('t')
                .long("track")
                .help("Target: Track")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["artist", "album"])
                .requires("sync"),
        )
        .arg(
            Arg::new("move")
                .short('m')
                .long("move")
                .help("Move files into repository")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["copy", "symlink"])
                .requires("update"),
        )
        .arg(
            Arg::new("copy")
                .short('c')
                .long("copy")
                .help("Copy files into repository")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["move", "symlink"])
                .requires("update"),
        )
        .arg(
            Arg::new("symlink")
                .long("symlink")
                .help("Create symlinks in repository")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["move", "copy"])
                .requires("update"),
        )
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .help("Search for music")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("info")
                .short('i')
                .long("info")
                .help("Display detailed information")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List items")
                .action(ArgAction::SetTrue)
                .requires("query"),
        )
        .arg(
            Arg::new("validate-local")
                .long("validate-local")
                .help("Validate local music repository")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("validate-remote")
                .long("validate-remote")
                .help("Validate remote music sources")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .help("Open configuration file in default editor")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .help("Specify audio format (flac, mp3, opus, etc.)")
                .value_name("FORMAT")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("quality")
                .short('q')
                .long("quality")
                .help("Specify quality/bitrate")
                .value_name("QUALITY")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("refresh")
                .short('y')
                .long("refresh")
                .help("Refresh remote source cache")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("noconfirm")
                .long("noconfirm")
                .help("Do not ask for confirmation")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Be verbose")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("recursive")
                .long("recursive")
                .help("Process directories recursively")
                .action(ArgAction::SetTrue)
                .requires("update"),
        )
        .arg(
            Arg::new("targets")
                .help("Target items (artists, albums, tracks, or paths)")
                .action(ArgAction::Append)
                .num_args(0..),
        )
}

pub fn handle_matches(matches: &ArgMatches) {
    // Handle standalone operations first
    if matches.get_flag("config") {
        open_config();
        return;
    }

    if matches.get_flag("validate-local") {
        validate_local_repo(matches.get_flag("verbose"));
        return;
    }

    if matches.get_flag("validate-remote") {
        validate_remote_repo(matches.get_flag("verbose"));
        return;
    }

    // Determine primary operation
    let operation = if matches.get_flag("sync") {
        "sync"
    } else if matches.get_flag("query") {
        "query"
    } else if matches.get_flag("remove") {
        "remove"
    } else if matches.get_flag("update") {
        "update"
    } else {
        eprintln!("Error: No operation specified");
        eprintln!("Use -S (download), -Q (query), -R (remove), -U (update), or --config/--validate-*");
        process::exit(1);
    };

    let verbose = matches.get_flag("verbose");
    let noconfirm = matches.get_flag("noconfirm");

    // Get targets if provided
    let targets: Vec<&String> = matches
        .get_many::<String>("targets")
        .unwrap_or_default()
        .collect();

    match operation {
        "sync" => handle_sync(matches, &targets, verbose, noconfirm),
        "query" => handle_query(matches, &targets, verbose),
        "remove" => handle_remove(matches, &targets, verbose, noconfirm),
        "update" => handle_update(matches, &targets, verbose, noconfirm),
        _ => unreachable!(),
    }
}

pub fn handle_sync(matches: &ArgMatches, targets: &[&String], verbose: bool, noconfirm: bool) {
    let artist = matches.get_flag("artist");
    let album = matches.get_flag("album");
    let track = matches.get_flag("track");
    let search = matches.get_flag("search");
    let info = matches.get_flag("info");
    let refresh = matches.get_flag("refresh");
    let format = matches.get_one::<String>("format");
    let quality = matches.get_one::<String>("quality");

    if verbose {
        println!("Operation: Sync (Download)");
    }

    if refresh {
        println!("Refreshing remote source cache...");
    }

    if search {
        if targets.is_empty() {
            eprintln!("Error: No search term specified");
            process::exit(1);
        }
        let target_type = if artist {
            "artists"
        } else if album {
            "albums"
        } else if track {
            "tracks"
        } else {
            "all"
        };
        println!("Searching for {}: {:?}", target_type, targets);
        return;
    }

    if info {
        if targets.is_empty() {
            eprintln!("Error: No target specified");
            process::exit(1);
        }
        let target_type = if artist {
            "artist"
        } else if album {
            "album"
        } else if track {
            "track"
        } else {
            "item"
        };
        println!("Getting info for {}: {:?}", target_type, targets);
        return;
    }

    if targets.is_empty() {
        eprintln!("Error: No targets specified");
        process::exit(1);
    }

    // Determine download type
    let download_type = if artist {
        "artist discography"
    } else if album {
        "album"
    } else if track {
        "track"
    } else {
        eprintln!("Error: No target type specified (use -A for artist, -a for album, -t for track)");
        process::exit(1);
    };

    println!("Downloading {} for: {:?}", download_type, targets);

    if let Some(fmt) = format {
        println!("Format: {}", fmt);
    }

    if let Some(qual) = quality {
        println!("Quality: {}", qual);
    }

    if !noconfirm {
        println!("Proceed with download? [Y/n]");
    }
}

pub fn handle_query(matches: &ArgMatches, targets: &[&String], verbose: bool) {
    let list = matches.get_flag("list");
    let search = matches.get_flag("search");
    let info = matches.get_flag("info");

    if verbose {
        println!("Operation: Query (Local Library)");
    }

    if list {
        println!("Listing local music library...");
    } else if search {
        if targets.is_empty() {
            eprintln!("Error: No search term specified");
            process::exit(1);
        }
        println!("Searching local library for: {:?}", targets);
    } else if info {
        if targets.is_empty() {
            eprintln!("Error: No target specified");
            process::exit(1);
        }
        println!("Getting local info for: {:?}", targets);
    } else if !targets.is_empty() {
        println!("Querying local library for: {:?}", targets);
    } else {
        println!("Listing local music library...");
    }
}

pub fn handle_remove(matches: &ArgMatches, targets: &[&String], verbose: bool, noconfirm: bool) {
    if verbose {
        println!("Operation: Remove");
    }

    if targets.is_empty() {
        eprintln!("Error: No targets specified");
        process::exit(1);
    }

    println!("Removing from library: {:?}", targets);

    if !noconfirm {
        println!("Proceed with removal? [Y/n]");
    }
}

pub fn handle_update(matches: &ArgMatches, targets: &[&String], verbose: bool, noconfirm: bool) {
    let move_files = matches.get_flag("move");
    let copy_files = matches.get_flag("copy");
    let symlink_files = matches.get_flag("symlink");
    let recursive = matches.get_flag("recursive");

    if verbose {
        println!("Operation: Update (Import to Repository)");
    }

    if targets.is_empty() {
        eprintln!("Error: No source paths specified");
        process::exit(1);
    }

    let operation = if move_files {
        "Moving"
    } else if copy_files {
        "Copying"
    } else if symlink_files {
        "Symlinking"
    } else {
        eprintln!("Error: No operation specified (use -m for move, -c for copy, -s for symlink)");
        process::exit(1);
    };

    println!("{} files into repository from: {:?}", operation, targets);

    if recursive {
        println!("Recursive mode enabled");
    }

    if !noconfirm {
        println!("Proceed with {}? [Y/n]", operation.to_lowercase());
    }
}

pub fn open_config() {
    println!("Opening configuration file in default editor...");
    // In real implementation, would open config file
    println!("Config path: ~/.config/flacman/flacman.conf");
}

pub fn validate_local_repo(verbose: bool) {
    println!("Validating local music repository...");
    if verbose {
        println!("Checking file integrity, metadata, and directory structure...");
    }
    println!("Validation complete: OK");
}

pub fn validate_remote_repo(verbose: bool) {
    println!("Validating remote music sources...");
    if verbose {
        println!("Checking connectivity and API status...");
    }
    println!("Validation complete: OK");
}