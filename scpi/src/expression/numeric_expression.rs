//! # 8.3.1 Numeric Expression
//! A numeric expression is a collection of terms which evaluates to a trace, number, array, or
//! other data element.

pub enum NumericOperator {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/` or `DIV`
    Div,
    /// `^`
    Exp,
    /// `MOD`
    Modulus,
    /// `OR`
    Or,
    /// `AND`
    And,
    /// `EXOR`
    Exor,
}

pub enum UnaryNumericOperator {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `NOT`
    Not,
}
