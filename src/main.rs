mod args;
use snowc::{Scanner, parse, type_check, Expr, Interpreter};
#[derive(Debug)]
enum CompilerError {
    NoFileGive,
    Parse(snowc::Error),
    Type(Vec<String>),
}

impl From<snowc::Error> for CompilerError {
    fn from(value: snowc::Error) -> Self {
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
                eprintln!("{token}");
            }
        }
        Ok(src)
    }

}

fn debug_ast(flag: bool) -> impl FnOnce(Vec<Expr>) -> Result<Vec<Expr>, CompilerError> {
    move |ast| {
        if flag {
            for node in ast.iter() {
                eprintln!("{node}");
            }
        }
        Ok(ast)
    }

}

fn handle_compiler_errors(error: CompilerError) {
}

fn main() {
    let setting = args::cargs();
    if setting.debug_graph {
        unimplemented!("graphviz is not working just yet");
    }
    let ast = setting
        .filename
        .and_then(|filename| {
            std::fs::read_to_string(filename).ok()
        }).ok_or(CompilerError::NoFileGive)
        .and_then(debug_tokens(setting.debug_token))
        .and_then(|src| {
            timer("Parse", ||parse(Scanner::new(&src)))
                .map_err(Into::into)
        })
        .and_then(debug_ast(setting.debug_ast))
        .and_then(|ast| {
            if !setting.option_no_type_check {
                timer("Type Check", || type_check(&ast)).map_err(Into::<CompilerError>::into)?;
            }
            Ok(ast)
        })
        .map_or_else(handle_compiler_errors, Interpreter::new);
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
