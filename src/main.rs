use std::{collections::HashMap, path::PathBuf};

use luau_ast_rs::ast::Comment;

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
pub fn populate_runtime_fns(runtime: &mlua::Lua) {
    let globals = runtime.globals();

    globals
        .set(
            "os_shell",
            runtime
                .create_function(|runtime, (cmd, capture_out): (String, bool)| {
                    if capture_out {
                        let output = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(cmd)
                            .output()
                            .unwrap();

                        let output_table = runtime.create_table().unwrap();
                        output_table
                            .set("out", String::from_utf8(output.stdout).unwrap().trim_end())
                            .unwrap();
                        output_table
                            .set("err", String::from_utf8(output.stderr).unwrap().trim_end())
                            .unwrap();
                        output_table
                            .set("status", output.status.code().unwrap())
                            .unwrap();

                        Ok(output_table)
                    } else {
                        let status = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(cmd)
                            .status()
                            .unwrap();

                        let output_table = runtime.create_table().unwrap();
                        output_table.set("status", status.code().unwrap()).unwrap();

                        Ok(output_table)
                    }
                })
                .unwrap(),
        )
        .unwrap();
    globals
        .set(
            "os_exit",
            runtime
                .create_function::<_, (), _>(|_, code: i32| std::process::exit(code))
                .unwrap(),
        )
        .unwrap();
    globals
        .set(
            "cd",
            runtime
                .create_function(|_, path: String| {
                    std::env::set_current_dir(path).unwrap();
                    Ok(())
                })
                .unwrap(),
        )
        .unwrap();
    globals
        .set(
            "canonicalize",
            runtime
                .create_function(|_, path: String| {
                    Ok(std::fs::canonicalize(path)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned())
                })
                .unwrap(),
        )
        .unwrap();
    globals
        .set(
            "getenv",
            runtime
                .create_function(|_, key: String| Ok(std::env::var(key).unwrap_or_default()))
                .unwrap(),
        )
        .unwrap();
    globals
        .set(
            "setenv",
            runtime
                .create_function(|_, (key, value): (String, String)| {
                    std::env::set_var(key, value);
                    Ok(())
                })
                .unwrap(),
        )
        .unwrap();
    globals
        .set(
            "workdir",
            runtime
                .create_function(|_, ()| {
                    Ok(std::env::current_dir()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned())
                })
                .unwrap(),
        )
        .unwrap();

    globals.set("OS", std::env::consts::OS).unwrap();
}

#[derive(Clone, Debug)]
pub struct TaskMetadata {
    pub tags: Vec<String>,
}

impl TaskMetadata {
    pub fn try_from_comment(comment: &str) -> Option<Self> {
        // Comment format: "-- @task #tag1 #tag2 .. #tagN"

        if !comment.starts_with("-- @task") {
            return None;
        }

        let mut tags = vec![];
        for word in comment.split_whitespace() {
            if word.starts_with('#') {
                tags.push(word.to_string());
            }
        }

        Some(TaskMetadata { tags })
    }
}

pub fn parse_task_comments(code: &str) -> HashMap<String, TaskMetadata> {
    use luau_ast_rs::{
        ast::{FunctionDef, LocalFunctionDef, StmtStatus},
        parser::Parser,
    };

    let chunk = Parser::new(code)
        .parse()
        .unwrap_or_else(|err| panic!("Lua code does not parse, we should've failed by now {err}"));

    let mut result = HashMap::new();

    for item in chunk {
        match item {
            StmtStatus::Some(stmt, comments) => match stmt {
                luau_ast_rs::ast::Stmt::FunctionDef(FunctionDef { name, .. })
                | luau_ast_rs::ast::Stmt::LocalFunctionDef(LocalFunctionDef { name, .. }) => {
                    //println!("Function {name} found with comments:\n{comments:?}",);

                    // WIP: Here
                    let metadata = comments
                        .iter()
                        .filter_map(|x| match x {
                            Comment::Leading(comment) => {
                                if let Some(metadata) = TaskMetadata::try_from_comment(comment) {
                                    Some(metadata)
                                } else {
                                    None
                                }
                            }
                            Comment::Trailing(_) => None,
                        })
                        .collect::<Vec<_>>();

                    if metadata.len() > 1 {
                        eprintln!("Error: Multiple task comments found for function {name}");
                        std::process::exit(1);
                    }

                    if metadata.len() == 1 {
                        result.insert(name, metadata[0].clone());
                    }
                }
                _ => (),
            },
            StmtStatus::None => (),
            StmtStatus::PreAllocated => (),
            StmtStatus::Error(_) => (),
        }
    }

    result
}

pub fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() <= 1 {
        eprintln!("Error: Expected task name");
        std::process::exit(1);
    }

    let runtime = mlua::Lua::new();
    populate_runtime_fns(&runtime);

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

    let metadata = parse_task_comments(&lulfile_contents);

    let task_name = &args[1].replace('-', "_");

    // Get the metadata for this task
    let Some(_task_meta) = metadata.get(task_name) else {
        let hint = if runtime
            .globals()
            .get::<_, mlua::Function>(task_name.as_str())
            .is_ok()
        {
            "\nHint: Function exists in file, but is not annotated with -- @task."
        } else {
            ""
        };
        eprintln!("Error: Task not found: '{task_name}'{hint}");
        std::process::exit(1);
    };

    // Find function with the given task name and call it
    let Ok(task_fn) = runtime
        .globals()
        .get::<_, mlua::Function>(task_name.as_str())
    else {
        eprintln!("Error: Task not found: '{}'", task_name);
        std::process::exit(1);
    };

    let mut fn_args = vec![];
    for arg in &args[2..] {
        let Ok(lua_val) = runtime.load(arg).eval::<mlua::Value>() else {
            eprintln!("Error: Failed to evaluate argument: '{}'", arg);
            std::process::exit(1);
        };
        fn_args.push(lua_val);
    }

    match task_fn.call::<_, ()>(mlua::MultiValue::from_vec(fn_args)) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error: Task {task_name} failed: '{err}'");
            std::process::exit(1);
        }
    };
}
