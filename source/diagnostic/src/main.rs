use clap::Parser;
use miette::{Diagnostic, GraphicalReportHandler, SourceSpan};
use std::fs;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("{message}")]
struct ViraError {
    message: String,
    #[source_code]
    src: String,
    #[label("here")]
    span: SourceSpan,
}

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
    handler.render_report(&mut out, &err as &dyn Diagnostic)?;
    println!("{}", out);

    Ok(())
}

fn calculate_offset(src: &str, line: usize, column: usize) -> usize {
    let mut offset = 0;
    let mut current_line = 1;

    for ch in src.chars() {
        if current_line == line {
            if column == 1 {
                return offset;
            } else {
                offset += ch.len_utf8();
                let new_column = if ch == '\n' { 1 } else { column - 1 };
                if new_column == 1 {
                    return offset;
                }
            }
        } else {
            offset += ch.len_utf8();
            if ch == '\n' {
                current_line += 1;
            }
        }
    }
    offset
}
