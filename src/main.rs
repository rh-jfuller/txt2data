#[cfg(feature = "cli")]
use clap::Parser as ClapParser;
use std::process::ExitCode;

#[cfg(feature = "cli")]
#[derive(ClapParser)]
#[command(
    name = "txt2data",
    about = "Convert text to structured data using ixml grammars",
    version
)]
struct Cli {
    /// ixml grammar file path, or inline grammar with -e
    #[arg(short, long)]
    grammar: Option<String>,

    /// Inline grammar expression
    #[arg(short = 'e', long)]
    expr: Option<String>,

    /// Input file path (reads stdin if omitted)
    #[arg(short, long)]
    input: Option<String>,

    /// Output format
    #[arg(
        short,
        long,
        default_value = "json",
        value_parser = ["json", "xml", "yaml", "sql", "sexp"]
    )]
    format: String,
}

fn main() -> ExitCode {
    #[cfg(feature = "cli")]
    {
        let cli = Cli::parse();
        match run(&cli) {
            Ok(output) => {
                #[expect(clippy::print_stdout)]
                {
                    println!("{output}");
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                #[expect(clippy::print_stderr)]
                {
                    eprintln!("error: {e}");
                }
                ExitCode::FAILURE
            }
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        ExitCode::FAILURE
    }
}

#[cfg(feature = "cli")]
fn run(cli: &Cli) -> Result<String, String> {
    let grammar_src = match (&cli.grammar, &cli.expr) {
        (Some(path), _) => {
            std::fs::read_to_string(path).map_err(|e| format!("cannot read grammar file: {e}"))?
        }
        (_, Some(expr)) => expr.clone(),
        _ => {
            return Err("provide --grammar <file> or -e <grammar>".into());
        }
    };

    let input = if let Some(path) = &cli.input {
        std::fs::read_to_string(path).map_err(|e| format!("cannot read input file: {e}"))?
    } else {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("cannot read stdin: {e}"))?;
        buf
    };

    match cli.format.as_str() {
        "xml" => txt2data::parse_to_xml(&grammar_src, &input),
        "yaml" => txt2data::parse_to_yaml(&grammar_src, &input),
        "sql" => txt2data::parse_to_sql(&grammar_src, &input),
        "sexp" => txt2data::parse_to_sexp(&grammar_src, &input),
        _ => txt2data::parse_to_json(&grammar_src, &input),
    }
}
