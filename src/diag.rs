use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::lower::LowerError;

// commmon error type for the diagnostic printer
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

// convert LexError -> Diagnostic
impl From<LexError> for Diagnostic {
    fn from(err: LexError) -> Self {
        Diagnostic {
            message: err.message,
            line: err.line,
            col: err.col,
        }
    }
}

// convert ParseError -> Diagnostic
impl From<ParseError> for Diagnostic {
    fn from(err: ParseError) -> Self {
        Diagnostic {
            message: err.message,
            line: err.line,
            col: err.col,
        }
    }
}

// Convert LowerError -> Diagnostic
impl From<LowerError> for Diagnostic {
    fn from(err: LowerError) -> Self {
        Diagnostic {
            message: err.message,
            line: err.line,
            col: err.col,
        }
    }
}

pub fn render(d: &Diagnostic, src: &str, path: &str) -> String {
    let line = src.lines().nth(d.line.saturating_sub(1)).unwrap_or("");

    //nuber of digits in the line number
    let gutter_width = d.line.to_string().len();

    let printed_line = line.replace('\t', " ");

    let mut output = String::new();

    //header
    output.push_str(
        &format!("error: {}\n", d.message)
    );
    //location
    output.push_str(
        &format!(
            "  --> {}:{}:{}\n",
            path,
            d.line,
            d.col,
        )
    );
    //empty gutter line
    output.push_str(
        &format!(
            "{:width$} |\n",
            "",
            width = gutter_width
        )
    );

    // actual sourse line
    output.push_str(
        &format!(
            "{:>width$} | {}\n",
            d.line,
            printed_line,
            width = gutter_width
        )
    );

    // Caret
    output.push_str(
        &format!(
            "{:width$} | {}^\n",
            "",
            " ".repeat(d.col.saturating_sub(1)),
            width = gutter_width
        )
    );

    output
}