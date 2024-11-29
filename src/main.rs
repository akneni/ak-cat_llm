use std::{env, fs, path::Path, process, usize};

use clipboard::{ClipboardContext, ClipboardProvider};
use glob::glob;

fn expand_files(files: &[String]) -> Vec<String> {
    let mut result = Vec::new();

    for pattern in files {
        let path = Path::new(pattern);

        if path.is_file() {
            // It's a file, add to result
            result.push(path.to_string_lossy().into_owned());
        } else if path.is_dir() {
            // It's a directory, list files in the directory (non-recursive)
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            result.push(entry_path.to_string_lossy().into_owned());
                        }
                    }
                }
            }
        } else {
            // Treat it as a glob pattern
            match glob(pattern) {
                Ok(paths) => {
                    for path in paths {
                        if let Ok(path) = path {
                            if path.is_file() {
                                result.push(path.to_string_lossy().into_owned());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Invalid glob pattern '{}': {}", pattern, e);
                }
            }
        }
    }
    result
}

fn gen_payload(files: &CatLlmArgs) -> String {
    let mut payload = String::with_capacity(1000);

    for f in &files.filenames {
        let mut text = match fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: unable to open file {} => {}", f, e);
                continue;
            }
        };
        if let Some(lim) = files.limit {
            let mut updated_text = text
                .split("\n")
                .map(|s| s.to_string())
                .take(lim)
                .collect::<Vec<String>>()
                .join("\n");
            if text.len() > updated_text.len() {
                updated_text.push_str("\n.....");
            }
            text = updated_text;
        }
        text = format!(
            "\\\\ {}\n{}\n\\\\End of file {}",
            f,
            text,
            f,
        );
        payload.push_str(&text);
        payload.push_str("\n\n\n");
    }
    payload
}

struct CatLlmArgs {
    clipbord: bool,
    limit: Option<usize>,
    filenames: Vec<String>,
}

impl Default for CatLlmArgs {
    fn default() -> Self {
        Self{clipbord: false, limit: None, filenames: vec![]}
    }
}

impl CatLlmArgs {
    fn from(cli_args: Vec<String>) -> Self {
        let mut final_args = Self::default();

        let mut i = 2;
        while i < cli_args.len() {
            if cli_args[i] == "-cb" || cli_args[i] == "--clipboard" {
                final_args.clipbord = true;
            }
            else if cli_args[i] == "-l" || cli_args[i] == "--limit" {
                if i >= cli_args.len() - 1 {
                    eprintln!("No argument passed to `--limit`.");
                    std::process::exit(1);                    
                }
                let lim = match cli_args[i+1].parse::<usize>() {
                    Ok(l) => l,
                    Err(_) => {
                        eprintln!("Invalid value passed to `--limit`. It must be an unsigned integer.");
                        std::process::exit(1);
                    }
                };
                final_args.limit = Some(lim);
                i += 1;
            }
            else if cli_args[i].starts_with("-l=") {
                let lim = match (&cli_args[i]["-l=".len()..]).parse::<usize>() {
                    Ok(l) => l,
                    Err(_) => {
                        eprintln!("Invalid value passed to `-l`. It must be an unsigned integer.");
                        std::process::exit(1);
                    }
                };
                final_args.limit = Some(lim);
            }
            else if cli_args[i].starts_with("--limit=") {
                let lim = match (&cli_args[i]["--limit=".len()..]).parse::<usize>() {
                    Ok(l) => l,
                    Err(_) => {
                        eprintln!("Invalid value passed to `--limit`. It must be an unsigned integer.");
                        std::process::exit(1);
                    }
                };
                final_args.limit = Some(lim);
            }
            else {
                final_args.filenames.push(cli_args[i].to_string());
            }
            i += 1;
        }

        final_args
    }
}

fn handle_cat_llm(cli_args: Vec<String>) {
    let mut cli_args = CatLlmArgs::from(cli_args);
    cli_args.filenames = expand_files(&cli_args.filenames);

    let payload = gen_payload(&cli_args);
    if cli_args.clipbord {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.set_contents(payload)
            .unwrap();
        println!("set contents");
    }
    else {
        println!("{}", payload);
    }
}

fn main() {
    let file_paths: Vec<String> = env::args().collect();

    if file_paths.len() <= 1 {
        eprintln!("No command found");
        process::exit(1);
    }
    match file_paths[1].as_str() {
        "cat-llm" | "llm-cat" => {
            handle_cat_llm(file_paths);
        }
        _ => {
            eprintln!("unsupported command");
            process::exit(1);
        }
    }
}
