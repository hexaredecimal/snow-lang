use clap::{crate_description, crate_name, crate_version, Arg, ColorChoice, Command};

#[derive(Debug)]
pub enum Target {
    JS, 
    Java, 
    VM
}

#[derive(Debug, Default)]
pub struct Settings {
    pub filename: Option<String>,
    pub target: Option<Target>,
    pub debug_token: bool,
    pub debug_ast: bool,
    pub debug_graph: bool,
    pub option_no_type_check: bool,
    pub option_compile_string: bool,
    pub verbose: bool,
    pub run: bool,
}

pub fn cargs() -> Settings {
    let matches = Command::new(crate_name!())
        .color(ColorChoice::Always)
        .version(crate_version!())
        .author("Cowboy8625, Hexaredecimal (JS Backend)")
        .about(crate_description!())
        .arg(Arg::new("filename"))
        .arg(
            Arg::new("debug-token")
                .long("debug-token")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("Show Tokens as they are created"),
        )
        .arg(
            Arg::new("target")
                .long("target")
                .required(false)
                .action(clap::ArgAction::Set)
                .help("Set target [JS/JAVA/VM]"),
        )
        .arg(
            Arg::new("debug-ast")
                .long("debug-ast")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("Show Ast"),
        )
        .arg(
            Arg::new("debug-graph")
                .long("debug-graph")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("Turns AST into a visual graph"),
        )
        .arg(
            Arg::new("run")
                .long("run")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("Verbose output"),
        )
        .arg(
            Arg::new("dynamic")
                .long("dynamic")
                .short('d')
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("turn the language in to garbage"),
        )
        .arg(
            Arg::new("from_string")
                .long("string")
                .short('s')
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("takes in string to compile"),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("Verbose output"),
        )
        .get_matches();

    let mut setting = Settings::default();
    if let Some(filename) = matches.get_one::<String>("filename") {
        setting.filename = Some(filename.to_string());
    }

    if let Some(target) = matches.get_one::<String>("target") {
        setting.target = match target.to_uppercase().as_str() {
            "JS" => Some(Target::JS), 
            "JAVA" => Some(Target::Java),
            "VM" => Some(Target::VM), 
            _ => panic!("Invalid target select: {target}")
        }
    }
    setting.debug_token = *matches
        .get_one::<bool>("debug-token")
        .expect("debug-token failed");
    setting.debug_ast = *matches
        .get_one::<bool>("debug-ast")
        .expect("debug-ast failed");
    setting.debug_graph = *matches
        .get_one::<bool>("debug-graph")
        .expect("debug-graph failed");
    setting.option_no_type_check =
        *matches.get_one::<bool>("dynamic").expect("dynamic failed");
    setting.option_compile_string = *matches
        .get_one::<bool>("from_string")
        .expect("from_string failed");
    setting.verbose = *matches.get_one::<bool>("verbose").expect("verbose failed");
    setting.run = *matches.get_one::<bool>("run").expect("run failed");
    setting
}
