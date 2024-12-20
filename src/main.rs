mod args;
use args::Target;
use clap::error::Result;
use snowc::error::Error;
use snowc::{debug_program, gen_code, parse, walk, Expr, Machine, Scanner};
use snowc_repl::repl;
use snowc::js::js_gen_code;
use snowc::java::java_gen_code; 

#[derive(Debug)]
enum CompilerError {
    NoFileGive,
    Parse(Vec<Error>),
    Type(Vec<String>),
}

impl From<Vec<Error>> for CompilerError {
    fn from(value: Vec<Error>) -> Self {
        Self::Parse(value)
    }
}

impl From<Vec<String>> for CompilerError {
    fn from(value: Vec<String>) -> Self {
        Self::Type(value)
    }
}

fn debug_tokens(flag: bool) -> impl FnOnce(String) -> Result<String, CompilerError> {
    move |src: String| {
        if flag {
            for token in Scanner::new(&src) {
                eprintln!("{token:?}");
            }
        }
        Ok(src)
    }
}

fn debug_ast(flag: bool) -> impl FnOnce(Vec<Expr>) -> Result<Vec<Expr>, CompilerError> {
    move |ast| {
        if flag {
            for node in ast.iter() {
                eprintln!("{node:#?}");
            }
        }
        Ok(ast)
    }
}

fn handle_compiler_errors(filename: impl Into<String>) -> impl FnOnce(CompilerError) {
    move |error_type| match error_type {
        CompilerError::Parse(ref errors) => {
            let filename = filename.into();
            let src = std::fs::read_to_string(&filename)
                .expect("failed to get file source for error report");
            for error in errors.iter() {
                let msg = error.report(&filename, &src);
                eprintln!("{msg}");
            }
        }
        CompilerError::Type(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
        }
        CompilerError::NoFileGive => {
            println!("");
        }
    }
}

fn get_src(flag: bool) -> impl FnOnce(String) -> Result<String, CompilerError> {
    move |filename| {
        if flag {
            return Ok(filename);
        }
        std::fs::read_to_string(filename)
            .ok()
            .ok_or(CompilerError::NoFileGive)
    }
}

fn main() {
    let setting = args::cargs();
    if setting.debug_graph {
        unimplemented!("graphviz is not working just yet");
    }
    setting
        .filename
        .clone()
        .ok_or_else(|| {
            let _ = repl();
            CompilerError::NoFileGive
        })
        .and_then(get_src(setting.option_compile_string))
        .and_then(debug_tokens(setting.debug_token))
        .and_then(|src| timer("Parsing", || parse(&src)).map_err(Into::into))
        .and_then(debug_ast(setting.debug_ast))
        // .and_then(|ast| {
        //     if !setting.option_no_type_check {
        //         timer("Type Checking", || type_check(&ast))
        //             .map_err(Into::<CompilerError>::into)?;
        //     }
        //     Ok(ast)
        // })
        // .map_or_else(
        //     handle_compiler_errors(setting.filename.clone().unwrap_or_default()),
        //     |ast| {
        //         let msg = format_compiler_message("Running");
        //         let filename = setting.filename.unwrap_or_default();
        //         eprintln!("{msg} {filename}");
        //         let result = walk(&ast);
        //         match result {
        //             Ok(_) => {}
        //             Err(errors) => {
        //                 let src =
        //                     get_src(setting.option_compile_string)(filename.clone())
        //                         .expect("failed to get file source for error report");
        //                 for err in errors.iter() {
        //                     let msg = err.report(&filename, &src);
        //                     eprintln!("{msg}");
        //                 }
        //             }
        //         }
        //     },
        // );
        .map_or_else(
            handle_compiler_errors(setting.filename.clone().unwrap_or_default()),
            |ast| {
                let program = timer("Codegen", || -> Result<String, CompilerError> {
                    if let Some(target) = setting.target {
                        let program = match target {
                            Target::JS => js_gen_code(&ast).unwrap(), 
                            Target::Java => java_gen_code(&ast).unwrap(),
                            Target::VM => gen_code(&ast)
                        };
                        Ok(program)
                    } else {
                        Ok(gen_code(&ast))
                    }
                })
                .unwrap();
                if setting.verbose {
                    println!("{program}");
                }

                if setting.run {
                    let msg = format_compiler_message("Running");
                    let filename = setting.filename.unwrap_or_default();
                    eprintln!("{msg} {filename}");
                    use rquickjs::{
                        CatchResultExt, Context, Function, Object, Result, Runtime, Value,
                    };

                    let rt = Runtime::new().unwrap();
                    let ctx = Context::full(&rt).unwrap();

                    let _ = ctx.with(|ctx| -> Result<()> {
                        let global = ctx.globals();
                        global.set(
                            "__print",
                            Function::new(ctx.clone(), print)?.with_name("__print")?,
                        )?;
                        ctx.eval::<(), _>(
                            r#"
                                    globalThis.console = {
                                        log(...v) {
                                            globalThis.__print(`${v.join(" ")}`)
                                        }
                                    }
                                "#,
                        )
                        .unwrap();

                        let console: Object = global.get("console")?;
                        let js_log: Function = console.get("log")?;
                        match ctx.eval::<Value, _>(program.as_bytes()).catch(&ctx) {
                            Ok(ret) => match js_log.call::<(Value<'_>,), ()>((ret,)) {
                                Err(err) => {
                                    println!("{err}")
                                }
                                Ok(_) => {}
                            },
                            Err(err) => {
                                println!("{err}");
                            }
                        }
                        Ok(())
                    });
                }

                /*debug_program(&program);
                let mut vm = Machine::new(program, false);
                vm.run();*/
            },
        );
}

fn print(s: String) {
    println!("{s}");
}

fn timer<O, E, F>(msg: impl Into<String>, func: F) -> Result<O, E>
where
    F: FnOnce() -> Result<O, E>,
{
    let start = std::time::Instant::now();
    let out = func();
    let now = std::time::Instant::now();
    let time = (now - start).as_secs_f64();
    let msg = format_compiler_message(msg);
    eprintln!("{msg} {time}s");
    out
}

fn format_compiler_message(msg: impl Into<String>) -> String {
    let msg = msg.into();
    let w = msg.len() + (15 - msg.len());
    let msg = format!("{:>w$}", msg);
    msg
}
