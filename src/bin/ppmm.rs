use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const CONFIG_FILE: &str = "ppmm.toml";

fn print_help() {
    println!("PPMM - Python Project Manager");
    println!("Usage: ppmm <command> [options]");
    println!();
    println!("Commands:");
    println!("  init <name>     Initialize a Python project");
    println!("  add <pkg> [ver] Add a package dependency");
    println!("  rm <pkg>        Remove a package dependency");
    println!("  list            List dependencies");
    println!("  install         Install all dependencies (via pip)");
    println!("  use <interp>    Set Python interpreter (e.g. 'rustpy')");
    println!("  run [script]    Run a Python script with rustpy");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" || args[1] == "help" {
        print_help();
        return;
    }

    if args[1] == "--version" || args[1] == "-V" {
        println!("ppmm 3.1.6");
        return;
    }

    match args[1].as_str() {
        "init" | "--init" => cmd_init(&args),
        "add" | "--add" => cmd_add(&args),
        "rm" | "remove" | "--rm" => cmd_remove(&args),
        "list" | "ls" | "--list" => cmd_list(),
        "install" | "--install" => cmd_install(),
        "use" => cmd_use(&args),
        "run" | "--run" => cmd_run(&args),
        _ => {
            eprintln!("Unknown command: '{}'. Use 'ppmm --help'.", args[1]);
        }
    }
}

struct Config {
    name: String,
    version: String,
    interpreter: String,
    deps: BTreeMap<String, String>,
}

fn read_config() -> Option<Config> {
    let content = fs::read_to_string(CONFIG_FILE).ok()?;
    let mut name = String::new();
    let mut version = String::new();
    let mut interpreter = String::new();
    let mut deps = BTreeMap::new();
    let mut section = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.trim_matches('[').trim_matches(']').to_string();
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let val = trimmed[eq_pos + 1..].trim().trim_matches('"').to_string();
            match section.as_str() {
                "project" if key == "name" => name = val,
                "project" if key == "version" => version = val,
                "project" if key == "interpreter" => interpreter = val,
                "dependencies" => {
                    deps.insert(key, val);
                }
                _ => {}
            }
        }
    }
    Some(Config {
        name,
        version,
        interpreter,
        deps,
    })
}

fn write_config(cfg: &Config) {
    write_config_at(CONFIG_FILE, cfg);
}

fn cmd_init(args: &[String]) {
    let name = args.get(2).map(|s| s.as_str()).unwrap_or("my_project");
    let path = Path::new(name);
    if path.exists() {
        eprintln!("Directory '{}' already exists", name);
        return;
    }
    fs::create_dir_all(path.join("src")).expect("Failed to create src directory");
    fs::create_dir_all(path.join("tests")).expect("Failed to create tests directory");
    let main_py = format!(
        "def main():\n    print(\"Hello from {}!\")\n\nif __name__ == \"__main__\":\n    main()\n",
        name
    );
    fs::write(path.join("src").join("main.py"), main_py).expect("Failed to write main.py");
    let cfg = Config {
        name: name.to_string(),
        version: "0.1.0".to_string(),
        interpreter: String::new(),
        deps: BTreeMap::new(),
    };
    write_config_at(path.join(CONFIG_FILE).to_str().unwrap(), &cfg);
    println!("Created Python project '{}'", name);
}

fn write_config_at(path: &str, cfg: &Config) {
    let mut out = String::new();
    out.push_str("[project]\n");
    out.push_str(&format!("name = \"{}\"\n", cfg.name));
    out.push_str(&format!("version = \"{}\"\n", cfg.version));
    if !cfg.interpreter.is_empty() {
        out.push_str(&format!("interpreter = \"{}\"\n", cfg.interpreter));
    }
    out.push('\n');
    out.push_str("[scripts]\nstart = \"src/main.py\"\n\n");
    out.push_str("[dependencies]\n");
    for (pkg, ver) in &cfg.deps {
        out.push_str(&format!("{} = \"{}\"\n", pkg, ver));
    }
    fs::write(path, out).expect("Failed to write ppmm.toml");
}

fn cmd_add(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: ppmm add <package> [version]");
        return;
    }
    let pkg = &args[2];
    let version = args.get(3).map(|s| s.as_str()).unwrap_or("*");
    let mut cfg = read_config().unwrap_or_else(|| {
        eprintln!("No ppmm.toml found in current directory");
        std::process::exit(1);
    });
    cfg.deps.insert(pkg.clone(), version.to_string());
    write_config(&cfg);
    println!("Added '{}' (version {})", pkg, version);
}

fn cmd_remove(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: ppmm rm <package>");
        return;
    }
    let pkg = &args[2];
    let mut cfg = read_config().unwrap_or_else(|| {
        eprintln!("No ppmm.toml found in current directory");
        std::process::exit(1);
    });
    if cfg.deps.remove(pkg.as_str()).is_some() {
        write_config(&cfg);
        println!("Removed '{}'", pkg);
    } else {
        eprintln!("Package '{}' not found in dependencies", pkg);
    }
}

fn cmd_list() {
    let cfg = match read_config() {
        Some(c) => c,
        None => {
            eprintln!("No ppmm.toml found.");
            return;
        }
    };
    if cfg.deps.is_empty() {
        println!("No dependencies.");
    } else {
        println!("Dependencies for '{}':", cfg.name);
        for (name, version) in &cfg.deps {
            println!("  {} = {}", name, version);
        }
    }
}

fn cmd_use(args: &[String]) {
    let interp = match args.get(2) {
        Some(s) => s.as_str(),
        None => {
            let cfg = read_config();
            match cfg.as_ref().and_then(|c| {
                if c.interpreter.is_empty() {
                    None
                } else {
                    Some(c.interpreter.as_str())
                }
            }) {
                Some(i) => {
                    println!("Using interpreter: {}", i);
                    return;
                }
                None => {
                    eprintln!("Usage: ppmm use <interpreter>");
                    return;
                }
            }
        }
    };
    let mut cfg = read_config().unwrap_or_else(|| {
        eprintln!("No ppmm.toml found in current directory");
        std::process::exit(1);
    });
    cfg.interpreter = interp.to_string();
    write_config(&cfg);
    println!("Set interpreter to '{}'", interp);
}

fn cmd_install() {
    let cfg = read_config().unwrap_or_else(|| {
        eprintln!("No ppmm.toml found in current directory");
        std::process::exit(1);
    });
    if cfg.deps.is_empty() {
        println!("No dependencies to install.");
        return;
    }
    let packages: Vec<&str> = cfg.deps.keys().map(|k| k.as_str()).collect();
    println!("Installing: {}", packages.join(" "));
    let status = Command::new("pip3")
        .arg("install")
        .args(&packages)
        .status()
        .expect("Failed to run pip3");
    if status.success() {
        println!("All packages installed.");
    } else {
        eprintln!(
            "pip3 install failed. Try 'pip install {}' manually.",
            packages.join(" ")
        );
    }
}

fn default_interpreter() -> String {
    env::current_exe()
        .ok()
        .and_then(|p| {
            let dir = p.parent()?;
            Some(if cfg!(windows) {
                dir.join("rustpy.exe")
            } else {
                dir.join("rustpy")
            })
        })
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "rustpy".to_string())
}

fn resolve_interpreter(name: &str) -> String {
    if name == "rustpy" {
        default_interpreter()
    } else {
        name.to_string()
    }
}

fn cmd_run(args: &[String]) {
    let script = args.get(2).map(|s| s.as_str()).unwrap_or("src/main.py");
    let bin = read_config()
        .and_then(|c| {
            if c.interpreter.is_empty() {
                None
            } else {
                Some(c.interpreter)
            }
        })
        .unwrap_or_else(default_interpreter);
    let bin = resolve_interpreter(&bin);
    let status = Command::new(&bin)
        .arg(script)
        .status()
        .expect("Failed to run interpreter");
    if !status.success() {
        eprintln!("Script '{}' exited with error", script);
    }
}
