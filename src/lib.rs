#[macro_use]
extern crate nom;

use nom::bytes::complete::tag;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_1() {
        assert!(parse_literal("3.14").is_ok());
    }

    #[test]
    fn literal_2() {
        assert!(parse_literal("34327689").is_ok());
    }

    #[test]
    fn several_expressions() {
        let expressions = [
            "7+4",
            "x^2",
            "3*y^2",
            "π",
            "x^2+y^2+x*y+x+5*y",
            "sqrt -1",
            "-7",
            "8--1",
            "-π",
            "7*(x+1)",
        ];

        expressions.iter().map(|&s| parse_expr(s)).for_each(
            |expr: Result<(&str, Expr), nom::Err<&str>>| {
                assert!(expr.is_ok());
            },
        );
    }
}

#[derive(Debug)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Function(FunctionID, Box<Expr>), // ln, sin, cos, sinh, cosh, ...etc.
    Exponent(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Abs(Box<Expr>),
    Var(String),
    Const(f64),
}

#[derive(Copy, Clone, Debug)]
enum FunctionID {
    Sqrt,
    Ln,
    Lb,
    Ld,
    Sin,
    Cos,
    Tan,
    Sinh,
    Cosh,
}

impl FunctionID {
    fn from_str(s: &str) -> FunctionID {
        use FunctionID::*;
        match s {
            "√" | "sqrt" => Sqrt,
            "ln" => Ln,
            "lb" => Lb,
            "ld" => Ld,
            "sin" => Sin,
            "cos" => Cos,
            "tan" => Tan,
            "sinh" => Sinh,
            "cosh" => Cosh,
            // TODO maybe, later use this entry point for user-defined functions?
            name => panic!("not a valid function name: {}", name),
        }
    }
}

named!(parse_expr<&str, Expr>,
    do_parse!(
        init: parse_expr_sub >>
        result: fold_many0!(
            preceded!(tag!("+"), parse_expr_sub),
            init,
            |acc, n| Expr::Add(Box::new(acc), Box::new(n))
        ) >> (result)
    )
);

named!(parse_expr_sub<&str, Expr>,
    do_parse!(
        init: parse_expr_mul >>
        result: fold_many0!(
            preceded!(tag!("-"), parse_expr_mul),
            init,
            |acc, n| Expr::Sub(Box::new(acc), Box::new(n))
        ) >> (result)
    )
);

named!(parse_expr_mul<&str, Expr>,
    do_parse!(
        init: parse_expr_div >>
        result: fold_many0!(
            preceded!(tag!("*"), parse_expr_div),
            init,
            |acc, n| Expr::Mul(Box::new(acc), Box::new(n))
        ) >> (result)
    )
);

named!(parse_expr_div<&str, Expr>,
    do_parse!(
        init: parse_expr_function >>
        result: fold_many0!(
            preceded!(tag!("/"), parse_expr_function),
            init,
            |acc, n| Expr::Div(Box::new(acc), Box::new(n))
        ) >> (result)
    )
);

named!(parse_expr_function<&str, Expr>,
    do_parse!(
        functions: many0!(
            alt!(
                tag!("√") |
                tag!("sqrt") |
                tag!("ln") |
                tag!("lb") |
                tag!("ld") |
                tag!("sin") |
                tag!("cos") |
                tag!("tan") |
                tag!("sinh") |
                tag!("cosh")
            )
        ) >>
        parameter: parse_expr_exponent >> (
            functions.iter().rev().fold(parameter, |acc, function| Expr::Function(
                    FunctionID::from_str(function),
                    Box::new(acc)
                    )
            )
        )
    )
);

named!(parse_expr_exponent<&str, Expr>,
    do_parse!(
        init: parse_signed_literal >>
        result: fold_many0!(
            preceded!(tag!("^"), parse_signed_literal),
            init,
            |acc, n| Expr::Exponent(Box::new(acc), Box::new(n))
        ) >> (result)
    )
);

named!(parse_signed_literal<&str, Expr>,
    do_parse!(
        sign: opt!(tag!("-")) >>
        literal: parse_literal >>
        (match sign {
            None => literal,
            Some(_) => Expr::Neg(Box::new(literal))
        })
    )
);

/// Parse
/// (<expr)
/// |<expr>|
/// (later) <expr, expr>
named!(parse_literal<&str, Expr>,
    alt!(
        delimited!(tag!("("), parse_expr, tag!(")")) |
        // map_res!(delimited!(tag!("|"), parse_expr, tag!("|")), |expr| Expr::Abs(Box::new(expr))) |

        map!(complete!(nom::double), |value| Expr::Const(value)) |
        // map!(nom::alphanumeric, |value| Expr::Var(value.to_string())) |
        map!(tag!("π"), |_| Expr::Const(3.141592653589793))
    )
);

named!(abc, map_res!(nom::digit, |&digits| digits));
