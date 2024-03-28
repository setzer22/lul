use std::path::PathBuf;

/// The lulfile is a file called lulfile, Lulfile or .lulfile in the current
/// directory, or any parent directory up to the root.
pub fn locate_lulfile() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().unwrap();
    loop {
        for candidate in ["lulfile", "Lulfile", ".lulfile", "lulfile.lua"] {
            let lulfile = current_dir.join(candidate);
            if lulfile.exists() {
                return Some(lulfile);
            }
        }
        if !current_dir.pop() {
            break;
        }
    }
    None
}

pub fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() <= 1 {
        eprintln!("Error: Expected task name");
        std::process::exit(1);
    }

    let runtime = mlua::Lua::new();

    let Some(lulfile) = locate_lulfile() else {
        eprintln!("Error: No lulfile found in current or parent directories.");
        std::process::exit(1);
    };

    let cwd = lulfile.parent().unwrap();
    std::env::set_current_dir(cwd).unwrap();

    let prelude = include_str!("../lua/prelude.lua");
    runtime.load(prelude).exec().unwrap();

    let lulfile_contents = std::fs::read_to_string(lulfile).unwrap();
    if let Err(err) = runtime.load(&lulfile_contents).exec() {
        eprintln!("Error: Failed to load lulfile\n{}", err);
        std::process::exit(1);
    }

    let task_name = &args[1];

    // Find function with the given task name and call it
    let Ok(task_fn) = runtime
        .globals()
        .get::<_, mlua::Function>(task_name.as_str())
    else {
        eprintln!("Error: Task not found: '{}'", task_name);
        std::process::exit(1);
    };

    let mut fn_args = vec![];
    for arg in &args[1..] {
        let Ok(lua_val) = runtime.load(arg).eval::<mlua::Value>() else {
            eprintln!("Error: Failed to evaluate argument: '{}'", arg);
            std::process::exit(1);
        };
        fn_args.push(lua_val);
    }

    match task_fn.call::<_, ()>(mlua::MultiValue::from_vec(fn_args)) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error: Task failed: '{}'", task_name);
            std::process::exit(1);
        }
    };
}
