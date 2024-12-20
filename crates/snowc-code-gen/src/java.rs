use snowc_parse::{App, Atom, Binary, Expr, TypeInfo, Unary};

pub fn java_gen_code(input: &Vec<Expr>) -> Option<String> {
    let mut stmts = vec![];
    // stmts.push("const print = globalThis.console.log".to_string()); //TODO: Implement a ffi
    stmts.push("import java.util.function.Function;\n".to_string());
    for expr in input {
        let stmt = match expr {
            Expr::Func(name, types, body, _) => {
                gen_function(name, &mut (types.clone()), body)
            }
            _ => todo!(),
        };
        stmts.push(stmt);
    }

    let program = stmts.join("\n");
    Some(program)
}

fn gen_function(name: &str, types: &mut Vec<TypeInfo>, body: &Expr) -> String {
    let count_types = types.len();
    let return_type = if count_types > 0 {
        type_to_gen(&types[count_types - 1])
    } else {
        type_to_gen(&TypeInfo::Custom("Object".to_string()))
    };

    let _body = gen_function_body(types, body);
    if name == "main" {
       format!("public static void {name}(String[] args) {{\n\t{_body};\n}}")
    } else {
        let arg = match body {
            Expr::Closure(_, _, _) => "Function<?, ?>", 
            _ => "Object"
        };
        format!("Function<{return_type}, {arg}> {name}() {{\n\treturn {_body};\n}}")
    }
}

fn gen_function_body(types: &mut Vec<TypeInfo>, body: &Expr) -> String {
    match body {
        Expr::Closure(_, _body, _) => {
            let count_types = types.len();
            if count_types > 0 {
                gen_expr(types, body, true)
            } else {
                gen_expr(types, body, true)
            }
        }
        _ => gen_expr(types, body, false),
    }
}

fn gen_expr(types: &mut Vec<TypeInfo>, expr: &Expr, ret: bool) -> String {
    match expr {
        Expr::Atom(atom) => gen_atom(atom.clone()),
        Expr::App(app) => gen_app(types, app.clone()),
        Expr::Enum(_, _, _) => todo!(),
        Expr::Closure(arg, body, _) => gen_closure(types, &arg, &body, ret),
        Expr::Array(exprs, _, _) => gen_array(types, exprs),
        Expr::Binary(bin) => gen_binary(types, bin),
        Expr::Unary(unary) => gen_unary(types, unary),
        Expr::IfElse(cond, if_true, if_false, _) => {
            gen_conditional(types, cond, if_true, if_false)
        }
        _ => todo!(),
    }
}

fn gen_conditional(
    types: &mut Vec<TypeInfo>,
    cond: &Expr,
    truth: &Expr,
    falsy: &Expr,
) -> String {
    let cond = gen_expr(types, cond, false);
    let truth = gen_expr(types, truth, false);
    let falsy = gen_expr(types, falsy, false);
    format!("({cond} ? {truth} : {falsy})")
}

fn gen_unary(types: &mut Vec<TypeInfo>, unary: &Unary) -> String {
    let op = &unary.op;
    let expr = gen_expr(types, &unary.expr, false);
    format!("({op} {expr})")
}

fn gen_binary(types: &mut Vec<TypeInfo>, binary: &Binary) -> String {
    let op = &binary.op;
    let left = gen_expr(types, &binary.left, false);
    let right = gen_expr(types, &binary.right, false);
    format!("({left} {op} {right})")
}

fn gen_array(types: &mut Vec<TypeInfo>, exprs: &[Expr]) -> String {
    let mut values = vec![];
    for value in exprs {
        values.push(gen_expr(types, value, false));
    }

    format!("[{}]", values.join(","))
}

fn gen_closure(types: &mut Vec<TypeInfo>, arg: &Expr, body: &Expr, ret: bool) -> String {
    let arg_type = gen_top_arg_type(types);
    let arg_value = gen_expr(types, &arg, ret);
    let _body = gen_expr(types, body, false);
    match body {
        Expr::Closure(_, _, _) => format!("({arg_type} {arg_value}) -> {_body}"), 
        _ => format!("({arg_type} {arg_value}) -> {_body}")
    }
}

fn gen_app(types: &mut Vec<TypeInfo>, app: App) -> String {
    let name = &app.name;
    let mut args = vec![];
    let size = app.args.len();
    for (i, arg) in app.args.into_iter().enumerate() {
        let expr = gen_expr(types, &arg, true);
        if i == 0 {
            args.push(format!("var {name}_{i} = {name}().apply({expr});"));
        } else if i < size -1 {
            let j = i - 1;
            args.push(format!("\tvar {name}_{i} = ((Function<?, ?>) {name}_{j}).apply({expr});"));
        } else {
            let j = i - 1;
            args.push(format!("\t((Function<?, ?>) {name}_{j}).apply({expr});"));
        }
    }

    format!("{}", args.join("\n"))
}

fn gen_atom(atom: Atom) -> String {
    match atom {
        Atom::Id(id, _, _) => id,
        Atom::Int(int, _, _) => format!("{int}"),
        Atom::Bool(bool, _, _) => format!("{bool}"),
        Atom::Char(char, _, _) => format!("{char}"),
        Atom::Float(float, _, _) => format!("{float}"),
        Atom::String(str, _, _) => format!("\"{str}\""),
    }
}

fn gen_top_arg_type(types: &mut Vec<TypeInfo>) -> String {
    let count_types = types.len();
    if count_types > 0 {
        type_to_gen(&types.remove(0))
    } else {
        type_to_gen(&TypeInfo::Custom("Object".to_string()))
    }
}

fn type_to_gen(_type: &TypeInfo) -> String {
    match _type {
        TypeInfo::Int => "Integer".to_string(),
        TypeInfo::Float => "Float".to_string(),
        TypeInfo::Bool => "Boolean".to_string(),
        TypeInfo::String => "String".to_string(),
        TypeInfo::Char => "Char".to_string(),
        TypeInfo::Array(inner) => format!("{}[]", type_to_gen(inner)),
        TypeInfo::Custom(value) => format!("{value}"),
    }
}
