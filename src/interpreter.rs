use super::error::{CResult, Error, ErrorKind};
use super::parser::{Atom, BuiltIn, Expr, Function, FunctionList};
use super::position::Spanned;
use std::io::Write;

pub fn evaluation(
    expr: &Expr,
    args: &[Spanned<Expr>],
    local: &mut FunctionList,
    global: &FunctionList,
) -> CResult<Expr> {
    match expr {
        Expr::Constant(_) => Ok(expr.clone()),
        Expr::Application(head, tail) => {
            let reduced_head = evaluation(&head.node, &tail, local, global)?;
            let reduced_tail = tail
                .into_iter()
                .map(|expr| Ok((evaluation(&expr.node, args, local, global)?, expr.span()).into()))
                .collect::<CResult<Vec<Spanned<Expr>>>>()?;
            match reduced_head {
                Expr::Constant(Atom::BuiltIn(bi)) => Ok(Expr::Constant(match bi {
                    BuiltIn::Plus if is_int(reduced_tail.first().clone()) => Atom::Int(
                        reduced_tail
                            .into_iter()
                            .map(get_int_from_expr)
                            .collect::<CResult<Vec<i32>>>()?
                            .into_iter()
                            .sum(),
                    ),
                    BuiltIn::Plus => Atom::Float(
                        reduced_tail
                            .into_iter()
                            .map(get_float_from_expr)
                            .collect::<CResult<Vec<f32>>>()?
                            .into_iter()
                            .sum(),
                    ),
                    BuiltIn::Mult if is_int(reduced_tail.first().clone()) => Atom::Int(
                        reduced_tail
                            .into_iter()
                            .map(get_int_from_expr)
                            .collect::<CResult<Vec<i32>>>()?
                            .into_iter()
                            .product(),
                    ),
                    BuiltIn::Mult => Atom::Float(
                        reduced_tail
                            .into_iter()
                            .map(get_float_from_expr)
                            .collect::<CResult<Vec<f32>>>()?
                            .into_iter()
                            .product(),
                    ),
                    BuiltIn::Mins if is_int(reduced_tail.first().clone()) => {
                        Atom::Int(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_int_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_int_from_expr)
                                .collect::<CResult<Vec<i32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold(fe, |a, b| a - b)
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::Mins => {
                        Atom::Float(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_float_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_float_from_expr)
                                .collect::<CResult<Vec<f32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold(fe, |a, b| a - b)
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::Div if is_int(reduced_tail.first().clone()) => {
                        Atom::Int(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_int_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_int_from_expr)
                                .collect::<CResult<Vec<i32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold(fe, |a, b| a / b)
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::Div => {
                        Atom::Float(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_float_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_float_from_expr)
                                .collect::<CResult<Vec<f32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold(fe, |a, b| a / b)
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::Eq if is_int(reduced_tail.first().clone()) => {
                        Atom::Boolean(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_int_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_int_from_expr)
                                .collect::<CResult<Vec<i32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold((fe, true), |(a, c), b| (b, c && a == b))
                                .1
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::Eq => {
                        Atom::Boolean(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_float_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_float_from_expr)
                                .collect::<CResult<Vec<f32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold((fe, true), |(a, c), b| (b, c && a == b))
                                .1
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::NEq if is_int(reduced_tail.first().clone()) => {
                        Atom::Boolean(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_int_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_int_from_expr)
                                .collect::<CResult<Vec<i32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold((fe, true), |(a, c), b| (b, c && a != b))
                                .1
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::NEq => {
                        Atom::Boolean(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_float_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_float_from_expr)
                                .collect::<CResult<Vec<f32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold((fe, true), |(a, c), b| (b, c && a != b))
                                .1
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::Not => {
                        if reduced_tail.len() != 1 {
                            return Err("Nothing on Rhs.".into());
                        } else {
                            Atom::Boolean(!get_bool_from_expr(
                                reduced_tail.first().cloned().unwrap(),
                            )?)
                        }
                    }
                    BuiltIn::Print => {
                        for i in reduced_tail.iter() {
                            print!("{}", i.node); // evaluation(&i.node, tail, local, global)?);
                        }
                        std::io::stdout().flush()?;
                        return Ok(reduced_tail[0].node.clone());
                    }
                    BuiltIn::PrintLn => {
                        for i in reduced_tail.iter() {
                            println!("{}", i.node); // evaluation(&i.node, &[], local, global)?);
                        }
                        return Ok(reduced_tail[0].node.clone());
                    }
                })),
                t => evaluation(&t, &reduced_tail, local, global),
            }
        }
        Expr::Local(name) => {
            let mut func = global
                .get(&name.node)
                .cloned()
                .or_else(|| local.get(&name.node).cloned())
                .ok_or_else(|| {
                    Error::new(
                        &format!("'{}' not found", name),
                        name.span(),
                        ErrorKind::Undefined,
                    )
                })?;
            let mut idx = 0;
            for _ in args
                .iter()
                .rev()
                .take_while(|a| func.bind_arg(a.node.clone()))
            {
                idx += 1;
            }
            let left_of_args = args[idx..].to_vec();
            func.local(local);
            let app = func.into_app(&left_of_args);
            // evaluation(&dbg!(app), &[], dbg!(local), dbg!(global))
            // evaluation(&app, &[], dbg!(local), global)
            evaluation(&app, &[], local, global)
        }
        Expr::Do(list_expr) => list_expr
            .clone()
            .iter()
            .map(|expr| evaluation(&expr.node, &[], local, global))
            .collect::<CResult<Vec<Expr>>>()?
            .last()
            .map(Clone::clone)
            .ok_or(Error::new(
                "nothing to return from do block",
                (list_expr.first(), list_expr.last()).into(),
                ErrorKind::EmptyReturn,
            )),
        Expr::Let(expr, body) => {
            // Let funcs not able to return
            for func in expr.iter() {
                match &func.node {
                    Expr::Function(name, prams, body) => local.insert(
                        name.node.clone(),
                        Function::new(
                            &name.node,
                            prams,
                            body.node.clone(),
                            (name.span(), body.span()).into(),
                        ),
                    ),
                    x => unreachable!(x),
                };
            }
            Ok(evaluation(&body.node, &[], local, global)?)
        }
        Expr::If(condition, body) => {
            let reduced_condition = evaluation(&condition.node, &[], local, global)?;
            let cond = get_bool_from_expr((reduced_condition, condition.span()).into())?;
            if cond {
                return evaluation(&body.node, &[], local, global);
            }
            Err(Error::new(
                "conditional may need a else statement",
                condition.span(),
                ErrorKind::EmptyReturn,
            ))
        }
        Expr::IfElse(condition, body, r#else) => {
            let reduced_condition = evaluation(&condition.node, &[], local, global)?;
            let cond = get_bool_from_expr((reduced_condition, condition.span()).into())?;
            if cond {
                return evaluation(&body.node, &[], local, global);
            } else {
                return evaluation(&r#else.node, &[], local, global);
            }
        }
        _ => unreachable!(),
    }
}

fn get_int_from_expr(e: Spanned<Expr>) -> CResult<i32> {
    if let Expr::Constant(Atom::Int(n)) = e.node {
        Ok(n)
    } else {
        Err(Error::new(
            &format!("{} is not 'Int'", e.node),
            e.span(),
            ErrorKind::TypeError,
        ))
    }
}

fn get_float_from_expr(e: Spanned<Expr>) -> CResult<f32> {
    if let Expr::Constant(Atom::Float(n)) = e.node {
        Ok(n)
    } else {
        Err(Error::new(
            &format!("{} is not 'Float'", e.node),
            e.span(),
            ErrorKind::TypeError,
        ))
    }
}

fn get_bool_from_expr(e: Spanned<Expr>) -> CResult<bool> {
    if let Expr::Constant(Atom::Boolean(b)) = e.node {
        Ok(b)
    } else {
        Err(Error::new(
            &format!("{} is not 'Boolean'", e.node),
            e.span(),
            ErrorKind::TypeError,
        ))
    }
}

fn is_int(oe: Option<&Spanned<Expr>>) -> bool {
    if let Some(v) = oe {
        return get_int_from_expr(v.clone()).is_ok();
    }
    false
}

// fn reduced_expr(
//     spanned: &Spanned<Expr>,
//     args: &[Spanned<Expr>],
//     local: &mut FunctionList,
//     global: &FunctionList,
// ) -> CResult<Spanned<Expr>> {
//     match &spanned.node {
//         Expr::Local(name) => {
//             let mut func = global
//                 .get(&name.node)
//                 .cloned()
//                 .or_else(|| local.get(&name.node).cloned())
//                 .ok_or_else(|| {
//                     Error::new(
//                         &format!("'{}' not found", name),
//                         name.span(),
//                         ErrorKind::Undefined,
//                     )
//                 })?;
//             let mut idx = 0;
//             for _ in args
//                 .iter()
//                 .rev()
//                 .take_while(|a| func.bind_arg(a.node.clone()))
//             {
//                 idx += 1;
//             }
//             let left_of_args = args[idx..].to_vec();
//             func.local(local);
//             Ok((func.into_app(&left_of_args), spanned.span()).into())
//             // global
//             //     .get(&name.node)
//             //     .cloned()
//             //     .or_else(|| local.get(&name.node).cloned())
//             //     .unwrap_or(spanned.clone());
//         }
//         Expr::Constant(_) => Ok(spanned.clone()),
//         _ => Ok((
//             evaluation(&spanned.node.clone(), &[], local, global).unwrap_or(spanned.node.clone()),
//             spanned.span(),
//         )
//             .into()),
//     }
// }

// fn get_curried_args<'a>(
//     spanned: &Spanned<Expr>,
//     args: &[Spanned<Expr>],
// ) -> (Spanned<Expr>, Vec<Spanned<Expr>>) {
//     match spanned.node.clone() {
//         Expr::Application(lhs, mut rhs) => {
//             rhs.extend_from_slice(args);
//             let (n, a) = get_curried_args(&lhs, &rhs);
//             (n, a)
//         }
//         _ => (spanned.clone(), args.to_vec()),
//     }
// }
