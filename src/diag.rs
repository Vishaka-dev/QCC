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

#[cfg(test)]
mod test {
    use super::*;

#[test]
fn render_tab_as_one_space() {
    let d = Diagnostic {
        message: "bad gate".to_string(),
        line: 1,
        col: 3,
    };

    let src = "\tHX";

    let actual = render(&d, src, "test.qc");

    let expected = "\
error: bad gate
  --> test.qc:1:3
  |
1 |  HX
  |   ^
";

    assert_eq!(actual, expected);
}

#[test]
fn render_error_past_last_line_does_not_panic() {
    let d = Diagnostic {
        message: "unexpected end of file".to_string(),
        line: 3,
        col: 1,
    };

    let src = "line one\nline two";

    let output = render(&d, src, "test.qc");

    assert!(output.contains("unexpected end of file"));
    assert!(output.contains("test.qc:3:1"));
}

#[test]
fn render_multi_digit_line_number() {
    let d = Diagnostic {
        message: "bad token".to_string(),
        line: 10,
        col: 3,
    };

    let src = "\
line1
line2
line3
line4
line5
line6
line7
line8
line9
abcdef";

    let actual = render(&d, src, "test.qc");

    let expected = "\
error: bad token
  --> test.qc:10:3
   |
10 | abcdef
   |   ^
";

    assert_eq!(actual, expected);
}

#[test]
fn render_one_line_error() {
    let d = Diagnostic {
        message: "something went wrong".to_string(),
        line: 1,
        col: 5,
    };

    let src = "abcdefgh";
    let path = "test.qc";

    let actual = render(&d, src, path);

    let expected = "\
error: something went wrong
  --> test.qc:1:5
  |
1 | abcdefgh
  |     ^
";

    assert_eq!(actual, expected);
}
}