use walkdir::WalkDir;
use dialoguer::{theme::SimpleTheme, MultiSelect, Input};
use indicatif::{ProgressBar, ProgressStyle};
use human_bytes::human_bytes;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use std::time::Duration;
use clap::Parser;
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use console::Term;

const TARGET_DIRS: &[&str] = &[
    "node_modules", // JS/TS
    "target",       // Rust
    "build",        // Java/Gradle/C++
    "dist",         // Web
    ".gradle",      // Gradle
    "vendor",       // PHP/Go
    "__pycache__",  // Python
    "bin", "obj",   // .NET
    ".dart_tool",   // Dart
    ".angular",     // Angular
    ".next",        // Next.js
    ".nuxt",        // Nuxt.js
];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: Option<String>,

    #[arg(short, long, default_value_t = 0)]
    min_size: u64,

    #[arg(long)]
    scan: bool,

    #[arg(long)]
    no_cache: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CandidateDir {
    path: PathBuf,
    size: u64,
}

fn is_target(name: &str) -> bool {
    TARGET_DIRS.contains(&name)
}

fn has_file(path: &Path, file_name: &str) -> bool {
    path.join(file_name).exists()
}

fn has_any_file(path: &Path, files: &[&str]) -> bool {
    files.iter().any(|f| path.join(f).exists())
}

fn has_file_with_extension(path: &Path, extension: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == extension {
                    return true;
                }
            }
        }
    }
    false
}

fn is_safe_to_delete(dir_name: &str, path: &Path) -> bool {
    let parent = match path.parent() {
        Some(p) => p,
        None => return false,
    };

    match dir_name {
         "node_modules" => has_file(parent, "package.json"),
         "target" => has_file(parent, "Cargo.toml"),
         "build" => has_any_file(parent, &["pom.xml", "build.gradle", "build.gradle.kts", "Makefile", "CMakeLists.txt", "angular.json"]),
         "dist" => has_any_file(parent, &["package.json", "angular.json", "tsconfig.json", "vite.config.js", "vite.config.ts"]),
         ".gradle" => has_any_file(parent, &["build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts"]),
         "vendor" => has_any_file(parent, &["composer.json", "go.mod", "Gemfile"]),
         "bin" | "obj" => has_file_with_extension(parent, "csproj") || has_file_with_extension(parent, "fsproj") || has_file_with_extension(parent, "sln"),
         "__pycache__" => true, // Usually safe to delete if found
         ".dart_tool" => has_file(parent, "pubspec.yaml"),
         ".angular" => has_file(parent, "angular.json"),
         ".next" => has_file(parent, "next.config.js") || has_file(parent, "next.config.ts"),
         ".nuxt" => has_file(parent, "nuxt.config.js") || has_file(parent, "nuxt.config.ts"),
         _ => false,
    }
}

fn calculate_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum()
}

fn get_cache_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "devpurge", "devpurge") {
        let cache_dir = proj_dirs.cache_dir();
        if !cache_dir.exists() {
            let _ = fs::create_dir_all(cache_dir);
        }
        return Some(cache_dir.join("scan_cache.json"));
    }
    None
}

fn load_cache(path: &Path) -> Option<Vec<CandidateDir>> {
    if let Ok(file) = fs::File::open(path) {
        if let Ok(candidates) = serde_json::from_reader(file) {
            return Some(candidates);
        }
    }
    None
}

fn save_cache(path: &Path, candidates: &[CandidateDir]) {
    if let Ok(file) = fs::File::create(path) {
        let _ = serde_json::to_writer(file, candidates);
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("DevPurge - Developer Dependency Cleaner");
    
    let path = match args.path {
        Some(p) => PathBuf::from(p),
        None => {
            let default_path = std::env::current_dir()?;
            let path_str: String = Input::with_theme(&SimpleTheme)
                .with_prompt("Enter path to scan")
                .default(default_path.to_string_lossy().to_string())
                .interact_text()?;
            PathBuf::from(path_str)
        }
    };
    
    if !path.exists() {
        eprintln!("Path does not exist!");
        return Ok(());
    }

    let cache_file_path = get_cache_path();
    let mut candidates: Vec<CandidateDir> = Vec::new();
    let mut from_cache = false;

    if !args.scan && !args.no_cache {
        if let Some(ref cache_path) = cache_file_path {
            if let Some(cached) = load_cache(cache_path) {
                 println!("Loaded {} results from cache.", cached.len());
                 candidates = cached.into_iter().filter(|c| c.path.exists()).collect();
                 from_cache = true;
            }
        }
    }

    if !from_cache {
        println!("Scanning {} for dependency folders... This may take a while.", path.display());
        
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap());
        spinner.enable_steady_tick(Duration::from_millis(100));

        let mut total_found_size = 0;
        let mut it = WalkDir::new(&path).into_iter();
        
        loop {
            let entry = match it.next() {
                None => break,
                Some(Err(_)) => continue,
                Some(Ok(entry)) => entry,
            };
            
            if entry.file_type().is_dir() {
                let file_name = entry.file_name().to_string_lossy();
                
                let display_path = entry.path().to_string_lossy();
                let char_count = display_path.chars().count();
                let short_display = if char_count > 50 {
                    let end_part: String = display_path.chars().skip(char_count - 47).collect();
                    format!("...{}", end_part)
                } else {
                    display_path.to_string()
                };
                spinner.set_message(format!("Scanning: {}", short_display));

                if is_target(&file_name) && is_safe_to_delete(&file_name, entry.path()) {
                    let size = calculate_size(entry.path());
                    total_found_size += size;
                    
                    candidates.push(CandidateDir {
                        path: entry.path().to_path_buf(),
                        size,
                    });
                    
                    it.skip_current_dir();
                }
            }
        }
        
        spinner.finish_and_clear();

        if !args.no_cache {
             if let Some(ref cache_path) = cache_file_path {
                 save_cache(cache_path, &candidates);
                 println!("Scan results cached.");
             }
        }
    }

    if candidates.is_empty() {
        println!("No dependency folders found.");
        return Ok(());
    }

    let min_bytes = args.min_size * 1024 * 1024;
    let original_count = candidates.len();
    
    if min_bytes > 0 {
        candidates.retain(|c| c.size >= min_bytes);
        println!("Filtered out {} folders smaller than {} MB.", original_count - candidates.len(), args.min_size);
    }
    
    if candidates.is_empty() {
        println!("No dependency folders found matching criteria.");
        return Ok(());
    }

    let total_size: u64 = candidates.iter().map(|c| c.size).sum();
    println!("Found {} folders. Total size: {}", candidates.len(), human_bytes(total_size as f64));

    candidates.sort_by(|a, b| b.size.cmp(&a.size));

    let term = Term::stdout();
    let _ = term.clear_screen();

    let term_cols = term.size().1 as usize;
    let max_width = if term_cols > 15 { term_cols - 15 } else { 60 };

    let options: Vec<String> = candidates.iter()
        .map(|c| {
            let size_str = human_bytes(c.size as f64);
            let raw_path = c.path.to_string_lossy();
            let full_str = format!("{} ({})", raw_path, size_str);
            
            if full_str.chars().count() > max_width {
                let extra_chars = 6;
                let available_space = max_width.saturating_sub(size_str.len() + extra_chars);
                
                if available_space < 10 {
                    let p_str = raw_path.to_string();
                    let chars_count = p_str.chars().count();
                    let end: String = p_str.chars().skip(chars_count.saturating_sub(max_width - size_str.len() - 5)).collect();
                    format!("...{} ({})", end, size_str)
                } else {
                    let keep = (available_space) / 2;
                    let p_str = raw_path.to_string();
                    let start: String = p_str.chars().take(keep).collect();
                    let end: String = p_str.chars().rev().take(keep).collect::<String>().chars().rev().collect();
                    format!("{}...{} ({})", start, end, size_str)
                }
            } else {
                full_str
            }
        })
        .collect();

    let defaults = vec![true; options.len()];

    println!("Select folders to DELETE (Up/Down to move, Space to toggle, Enter to confirm)");

    let selections = MultiSelect::with_theme(&SimpleTheme)
        .with_prompt("")
        .items_checked(&options.iter().zip(defaults.iter()).map(|(s, &b)| (s.as_str(), b)).collect::<Vec<_>>())
        .max_length(8)
        .clear(true)
        .interact()?;

    if selections.is_empty() {
        println!("No folders selected. Exiting.");
        return Ok(());
    }

    println!("\nSelected folders:");
    for &idx in &selections {
        println!("  {}", options[idx]);
    }

    let selected_count = selections.len();
    println!("\nAre you sure you want to delete {} folders? (type 'yes' to confirm)", selected_count);
    
    let confirmation: String = Input::new().interact_text()?;
    if confirmation.trim().to_lowercase() != "yes" {
        println!("Operation cancelled.");
        return Ok(());
    }

    println!("Deleting {} folders...", selected_count);
    
    let delete_bar = ProgressBar::new(selected_count as u64);
    delete_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("##-"));

    let mut reclaimed_space = 0;

    let mut deleted_paths = Vec::new();

    for &idx in &selections {
        let candidate = &candidates[idx];
        delete_bar.set_message(format!("Deleting {}", candidate.path.display()));
        
        if let Err(e) = fs::remove_dir_all(&candidate.path) {
            delete_bar.println(format!("Failed to delete {}: {}", candidate.path.display(), e));
        } else {
            reclaimed_space += candidate.size;
            deleted_paths.push(candidate.path.clone());
        }
        delete_bar.inc(1);
    }
    
    delete_bar.finish_with_message("Done!");
    
    if !args.no_cache && !deleted_paths.is_empty() {
        if let Some(ref cache_path) = cache_file_path {
            if let Some(mut full_cache) = load_cache(cache_path) {
                 full_cache.retain(|c| !deleted_paths.contains(&c.path));
                 save_cache(cache_path, &full_cache);
            }
        }
    }
    
    println!("Cleanup complete! Reclaimed space: {}", human_bytes(reclaimed_space as f64));
    
    Ok(())
}
