use super::Expr;
use super::FunctionList;
pub fn haskell_code_gen(funcs: FunctionList, _filename: &str) -> String {
    let mut code = String::new();
    for (name, func) in funcs.iter() {
        code.push_str(name);
        for pram in func.prams().iter() {
            code.push_str(&format!("{} ", pram));
        }
        code.push_str(" = ");
        code.push_str(&code_gen_expr(func.body()));
    }
    code
}

pub fn code_gen_expr(expr: Expr) -> String {
    match expr {
        Expr::Constant(atom) => atom.display(),
        Expr::Application(app, args) => format!(
            "{} {}",
            code_gen_expr(app.node),
            args.iter()
                .map(|arg| format!("{} ", code_gen_expr(arg.node.clone())))
                .collect::<String>()
        ),
        Expr::Function(_name, _prams, _body) => unreachable!(),
        Expr::Local(_name) => unimplemented!("Expr::Local is not implemented yet"),
        // FIXME: some how need to keep track of indention
        Expr::Do(block) => format!(
            "do\n{}",
            block
                .iter()
                .map(|arg| format!("    {}\n", code_gen_expr(arg.node.clone())))
                .collect::<String>()
        ),
        Expr::Let(_name, _body) => unimplemented!("Expr::Let is not implemented yet"),
        Expr::If(_condition, _body) => unimplemented!("Expr::If is not implemented yet"),
        Expr::IfElse(_condition, _body, _then) => {
            unimplemented!("Expr::IfElse is not implemented yet")
        }
    }
}
