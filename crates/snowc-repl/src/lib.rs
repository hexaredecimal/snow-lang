mod terminal;
use terminal::{Pos, Command, Terminal};
use crossterm::style::Stylize;

use anyhow::Result;

const PROMPT: &str = ":> ";
const WELCOME: &str = "snow-lang version 0.0.0\r\n";
const HELP_MESSAGE: &str = "Help Commands
:help             get this message
:exit | :quit     kill repl
:clear            clears screen
:scope            show what is in scope
:history          shows history from prompt
:load <filename>  loads a snow file
";

use snowc_tree_walker::{eval_expr_with_scope, Scope, Value};

pub fn repl() -> Result<()> {
    println!("{}\n{}", WELCOME, HELP_MESSAGE);
    let mut repl = Repl::new();
    let mut terminal = Terminal::new()?;
    let mut scope = Scope::default();

    terminal.print(PROMPT)?;
    terminal.flush()?;

    while repl.is_running() {

        let Some(command) = terminal.get_input() else {
            continue;
        };

        match command {
            Command::InsertChar(c) => repl.insert_char(c),
            Command::Backspace => backspace(&mut terminal, &mut repl)?,
            Command::Return => execute_return_command(&mut terminal, &mut repl, &mut scope)?,
            Command::Clear => repl.pos.y = 0,
            Command::Quit => repl.quit(),
        }


        if !matches!(command, Command::Return) {
            let mut s = scope.clone();
            match compile(&format!("{}  ", repl.input), &mut s) {
                Ok(Some(v)) => {
                    terminal.scroll_up_if_needed()?;
                    let y = terminal.y() + 1;
                    terminal.print_at(0, y, &v.to_string().grey().to_string())?;
                }
                _ => {
                    terminal.clear_from_cursor_down()?;
                }
            }
        }

        terminal.print(&format!("{PROMPT}{}", &repl.input))?;
        terminal.flush()?;

    }
    Ok(())
}

// Not sure what to return here
fn compile(input: &str, scope: &mut Scope) -> std::result::Result<Option<Value>, Vec<String>> {
    let lexer = |i| snowc_lexer::Scanner::new(i);
    let ast = match snowc_parse::parse(lexer(input)) {
        Ok(ast) => ast,
        Err(_) => match snowc_parse::parse_expr(lexer(input)) {
            Ok(ast) => vec![ast],
            Err(err) => return Err(err.into_iter().map(|x| x.report("snowc", input)).collect()),
        }
    };
    if ast.iter().any(|x| x.is_error()) {
        return Ok(None);
    }


    let mut results = vec![];
    for node in ast {
        match eval_expr_with_scope(&node, scope) {
            Ok(v) => results.push(v),
            Err(err) => return Err(vec![err.report("snowc", input)]),
        }
    }
    Ok(results.pop().flatten())
}

fn execute_return_command(terminal: &mut Terminal, repl: &mut Repl, scope: &mut Scope) -> Result<()> {
    if execute_builtin_repl_command(terminal, repl)? {
        return Ok(());
    }
    repl.input.push_str(" ");
    match compile(&repl.input, scope) {
        Ok(Some(v)) => {
            terminal.print(&v.to_string().yellow().to_string())?;
            terminal.new_line()?;
        }
        Err(errors) => {
            terminal.print(&errors.join("\n"))?;
        }
        _ => {},
    }
    repl.clear_input();
    Ok(())
}

fn execute_builtin_repl_command(terminal: &mut Terminal, repl: &mut Repl) -> Result<bool> {
    match repl.input.as_str() {
        ":exit" | ":quit" => {
            repl.quit();
            Ok(true)
        }
        ":clear" => {
            terminal.clear_screen()?;
            repl.pos.y = 0;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn backspace(terminal: &mut Terminal, repl: &mut Repl) -> Result<()> {
    repl.backspace();
    terminal.clear_line()?;
    Ok(())
}

struct Repl {
    input: String,
    running: bool,
    pos: Pos<usize>,
    // history: History,
}

impl Repl {
    fn new() -> Self {
        Self {
            input: String::new(),
            running: true,
            pos: Pos::default(),
        }
    }

    fn insert_char(&mut self, c: char) {
        self.input.insert(self.pos.x, c);
        self.pos.x += 1;
    }

    fn clear_input(&mut self) {
        self.pos.x = 0;
        self.input.clear();
    }

    fn backspace(&mut self) {
        if self.pos.x > 0 {
            self.pos.x -= 1;
            self.input.remove(self.pos.x);
        }
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn quit(&mut self) {
        self.running = false;
    }
}
