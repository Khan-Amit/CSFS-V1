//! CSFS Command-Line Tool

use std::path::PathBuf;
use csfs::Csfs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: csfs [format|write|read|advance] <path> ...");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "format" => {
            let path = PathBuf::from(&args[2]);
            let size_mb = args[3].parse().unwrap();
            Csfs::format(&path, size_mb).unwrap();
            println!("Formatted {} with {} MB ring", path.display(), size_mb);
        }
        "write" => {
            let path = PathBuf::from(&args[2]);
            let domain = args[3].parse().unwrap();
            let desire = args[4].parse().unwrap();
            let data = args[5].as_bytes();
            let mut csfs = Csfs::open(&path).unwrap();
            csfs.append(domain, desire, data).unwrap();
            println!("Written slice (domain={}, desire={})", domain, desire);
        }
        "read" => {
            let path = PathBuf::from(&args[2]);
            let offset = args[3].parse().unwrap();
            let mut csfs = Csfs::open(&path).unwrap();
            let data = csfs.read(offset).unwrap();
            println!("Data: {:?}", String::from_utf8_lossy(&data));
        }
        "advance" => {
            let path = PathBuf::from(&args[2]);
            let min_desire = args[3].parse().unwrap();
            let mut csfs = Csfs::open(&path).unwrap();
            let removed = csfs.advance_tail(min_desire).unwrap();
            println!("Removed {} slices", removed);
        }
        _ => eprintln!("Unknown command"),
    }
}
