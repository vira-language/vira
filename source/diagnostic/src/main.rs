use clap::Parser;
use miette::{Diagnostic, GraphicalReportHandler, SourceSpan};
use std::fs;
use std::fmt;

#[derive(Debug, Diagnostic)]
struct ViraError {
    message: String,
    #[source_code]
    src: String,
    #[label("here")]
    span: SourceSpan,
}

impl fmt::Display for ViraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ViraError {}

#[derive(Parser, Debug)]
#[command(version, about = "Vira Diagnostic Tool")]
struct Args {
    /// Path to the source file
    #[arg(short, long)]
    source: String,
    /// Error message
    #[arg(short, long)]
    message: String,
    /// Line number (1-based)
    #[arg(short, long)]
    line: usize,
    /// Column number (1-based)
    #[arg(short, long)]
    column: usize,
    /// Length of the span
    #[arg(short, long, default_value_t = 1)]
    length: usize,
}

fn main() -> miette::Result<()> {
    let args = Args::parse();
    let src = fs::read_to_string(&args.source).map_err(|e| miette::miette!("Failed to read source: {}", e))?;
    let offset = calculate_offset(&src, args.line, args.column);
    let span = SourceSpan::new(offset.into(), args.length.into());
    let err = ViraError {
        message: args.message,
        src,
        span,
    };
    let mut handler = GraphicalReportHandler::new();
    let mut out = String::new();
    handler.render_report(&mut out, &err as &dyn Diagnostic)
        .map_err(|e| miette::miette!("Failed to render report: {}", e))?;
    println!("{}", out);
    Ok(())
}

fn calculate_offset(src: &str, line: usize, column: usize) -> usize {
    let mut offset: usize = 0;
    let mut current_line = 1;

    let mut chars = src.chars().peekable();

    while current_line < line {
        if let Some(&ch) = chars.peek() {
            if ch == '\n' {
                current_line += 1;
            }
            offset += ch.len_utf8();
            chars.next();
        } else {
            return offset;
        }
    }

    let mut current_column = 1;
    while current_column < column {
        if let Some(&ch) = chars.peek() {
            if ch == '\n' {
                return offset;
            }
            offset += ch.len_utf8();
            chars.next();
            current_column += 1;
        } else {
            return offset;
        }
    }

    offset
}
