use crate::lexer::{Literal, TokenKind};
use crate::location::{Loc, Located};
use crate::stmt::Stmt;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(PartialEq, Clone)]
pub enum ExprKind {
    Literal(LitExpr),
    Function(Vec<Param>, Vec<Stmt>),
    Unary(UnOp, Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Logical(Box<Expr>, LogOp, Box<Expr>),
    Grouping(Box<Expr>),
    Comma(Box<Expr>, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
    Variable(String),
    Assign(String, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Get(Box<Expr>, String),
    Set(Box<Expr>, String, Box<Expr>),
    Array(Vec<Expr>),
    SubscriptGet(Box<Expr>, Box<Expr>),
    SubscriptSet(Box<Expr>, Box<Expr>, Box<Expr>),
    This,
    Super(String),
}

pub type Expr = Located<ExprKind>;
pub type Param = Located<String>;

pub trait Visitor<Res> {
    type Error;
    type Result = std::result::Result<Res, Self::Error>;

    fn visit_literal_expr(&mut self, literal: &LitExpr, loc: Loc) -> Self::Result;

    fn visit_function_expr(&mut self, params: &[Param], body: &[Stmt], loc: Loc) -> Self::Result;

    fn visit_unary_expr(&mut self, op: &UnOp, expr: &Expr, loc: Loc) -> Self::Result;

    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        op: &BinOp,
        right: &Expr,
        loc: Loc,
    ) -> Self::Result;

    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        op: &LogOp,
        right: &Expr,
        loc: Loc,
    ) -> Self::Result;

    fn visit_grouping_expr(&mut self, expr: &Expr, loc: Loc) -> Self::Result;

    fn visit_comma_expr(&mut self, left: &Expr, right: &Expr, loc: Loc) -> Self::Result;

    fn visit_cond_expr(&mut self, cond: &Expr, left: &Expr, right: &Expr, loc: Loc)
        -> Self::Result;

    fn visit_variable_expr(&mut self, name: &str, loc: Loc) -> Self::Result;

    fn visit_assign_expr(&mut self, name: &str, expr: &Expr, loc: Loc) -> Self::Result;

    fn visit_call_expr(&mut self, callee: &Expr, args: &[Expr], loc: Loc) -> Self::Result;

    fn visit_get_expr(&mut self, obj: &Expr, name: &str, loc: Loc) -> Self::Result;

    fn visit_set_expr(&mut self, obj: &Expr, name: &str, expr: &Expr, loc: Loc) -> Self::Result;

    fn visit_array_expr(&mut self, elements: &[Expr], loc: Loc) -> Self::Result;

    fn visit_subscript_get_expr(&mut self, obj: &Expr, index: &Expr, loc: Loc) -> Self::Result;

    fn visit_subscript_set_expr(
        &mut self,
        obj: &Expr,
        index: &Expr,
        expr: &Expr,
        loc: Loc,
    ) -> Self::Result;

    fn visit_this_expr(&mut self, loc: Loc) -> Self::Result;

    fn visit_super_expr(&mut self, method: &str, loc: Loc) -> Self::Result;
}

impl Expr {
    pub fn from_literal(literal: &Literal, loc: Loc) -> Self {
        use Literal::*;
        let kind = ExprKind::Literal(match literal {
            Integer(int) => LitExpr::Integer(*int),
            Float(float) => LitExpr::Float(*float),
            Str(string) => LitExpr::Str(string.clone()),
        });

        Expr::new(kind, loc)
    }

    pub fn integer(int: i64, loc: Loc) -> Self {
        Expr::new(ExprKind::Literal(LitExpr::Integer(int)), loc)
    }

    pub fn boolean(boolean: bool, loc: Loc) -> Self {
        Expr::new(ExprKind::Literal(LitExpr::Boolean(boolean)), loc)
    }

    pub fn nil(loc: Loc) -> Self {
        Expr::new(ExprKind::Literal(LitExpr::Nil), loc)
    }

    pub fn function(params: Vec<Param>, body: Vec<Stmt>, loc: Loc) -> Self {
        Expr::new(ExprKind::Function(params, body), loc)
    }

    pub fn unary(op: UnOp, right: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::Unary(op, Box::new(right)), loc)
    }

    pub fn binary(left: Expr, op: BinOp, right: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::Binary(Box::new(left), op, Box::new(right)), loc)
    }

    pub fn logical(left: Expr, op: LogOp, right: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::Logical(Box::new(left), op, Box::new(right)), loc)
    }

    pub fn grouping(expr: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::Grouping(Box::new(expr)), loc)
    }

    pub fn comma(left: Expr, right: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::Comma(Box::new(left), Box::new(right)), loc)
    }

    pub fn conditional(cond: Expr, left: Expr, right: Expr, loc: Loc) -> Self {
        Expr::new(
            ExprKind::Conditional(Box::new(cond), Box::new(left), Box::new(right)),
            loc,
        )
    }

    pub fn variable(name: &str, loc: Loc) -> Self {
        Expr::new(ExprKind::Variable(String::from(name)), loc)
    }

    pub fn assign(name: String, expr: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::Assign(name, Box::new(expr)), loc)
    }

    pub fn call(callee: Expr, args: Vec<Expr>, loc: Loc) -> Self {
        Expr::new(ExprKind::Call(Box::new(callee), args), loc)
    }

    pub fn get(obj: Expr, name: &str, loc: Loc) -> Self {
        Expr::new(ExprKind::Get(Box::new(obj), String::from(name)), loc)
    }

    pub fn set(obj: Box<Expr>, name: String, expr: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::Set(obj, name, Box::new(expr)), loc)
    }

    pub fn array(elements: Vec<Expr>, loc: Loc) -> Self {
        Expr::new(ExprKind::Array(elements), loc)
    }

    pub fn subscript_get(obj: Expr, index: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::SubscriptGet(Box::new(obj), Box::new(index)), loc)
    }

    pub fn subscript_set(obj: Box<Expr>, index: Box<Expr>, expr: Expr, loc: Loc) -> Self {
        Expr::new(ExprKind::SubscriptSet(obj, index, Box::new(expr)), loc)
    }

    pub fn this(loc: Loc) -> Self {
        Expr::new(ExprKind::This, loc)
    }

    pub fn super_expr(method: &str, loc: Loc) -> Self {
        Expr::new(ExprKind::Super(String::from(method)), loc)
    }

    pub fn accept<Vis, Res, Error>(&self, visitor: &mut Vis) -> Vis::Result
    where
        Vis: Visitor<Res, Error = Error>,
    {
        use ExprKind::*;
        match &self.kind {
            Literal(literal) => visitor.visit_literal_expr(literal, self.loc),
            Function(params, body) => visitor.visit_function_expr(params, body, self.loc),
            Unary(op, expr) => visitor.visit_unary_expr(op, expr, self.loc),
            Binary(left, op, right) => visitor.visit_binary_expr(left, op, right, self.loc),
            Logical(left, op, right) => visitor.visit_logical_expr(left, op, right, self.loc),
            Grouping(expr) => visitor.visit_grouping_expr(expr, self.loc),
            Comma(left, right) => visitor.visit_comma_expr(left, right, self.loc),
            Conditional(cond, left, right) => visitor.visit_cond_expr(cond, left, right, self.loc),
            Variable(name) => visitor.visit_variable_expr(name, self.loc),
            Assign(name, expr) => visitor.visit_assign_expr(name, expr, self.loc),
            Call(callee, args) => visitor.visit_call_expr(callee, args, self.loc),
            Get(obj, name) => visitor.visit_get_expr(obj, name, self.loc),
            Set(obj, name, expr) => visitor.visit_set_expr(obj, name, expr, self.loc),
            Array(elements) => visitor.visit_array_expr(elements, self.loc),
            SubscriptGet(obj, index) => visitor.visit_subscript_get_expr(obj, index, self.loc),
            SubscriptSet(obj, index, expr) => {
                visitor.visit_subscript_set_expr(obj, index, expr, self.loc)
            }
            This => visitor.visit_this_expr(self.loc),
            Super(method) => visitor.visit_super_expr(method, self.loc),
        }
    }
}

impl Default for ExprKind {
    fn default() -> Self {
        Self::Literal(LitExpr::Nil)
    }
}

impl Debug for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use ExprKind::*;
        let string = match &self {
            Literal(literal) => literal.to_string(),
            Function(params, body) => format!("(fun {:?} {:?})", params, body),
            Unary(operator, right) => parenthesize(operator.to_string(), &[right]),
            Binary(left, operator, right) => parenthesize(operator.to_string(), &[left, right]),
            Logical(left, operator, right) => parenthesize(operator.to_string(), &[left, right]),
            Grouping(expr) => parenthesize("group", &[expr]),
            Comma(left, right) => parenthesize("comma", &[left, right]),
            Conditional(cond, left, right) => parenthesize("?:", &[cond, left, right]),
            Variable(name) => format!("(var {})", name),
            Assign(name, expr) => format!("(= {} {:?})", name, expr),
            Call(callee, args) => format!("(call {:?} {:?})", callee, args),
            Get(obj, name) => format!("(get {:?} {})", obj, name),
            Set(obj, name, expr) => format!("(set {:?} {} {:?})", obj, name, expr),
            Array(elements) => format!("(array {:?})", elements),
            SubscriptGet(obj, index) => format!("(s-get {:?} {:?})", obj, index),
            SubscriptSet(obj, index, expr) => format!("(s-set {:?} {:?} {:?})", obj, index, expr),
            This => String::from("this"),
            Super(method) => format!("(super {})", method),
        };

        write!(f, "{}", string)
    }
}

fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
    let mut parts = vec![String::from("("), String::from(name)];

    for expr in exprs {
        parts.push(format!(" {:?}", expr));
    }

    parts.push(String::from(")"));

    parts.join("")
}

#[derive(PartialEq, Clone)]
pub enum LitExpr {
    Integer(i64),
    Float(f64),
    Str(String),
    Boolean(bool),
    Nil,
}

impl Display for LitExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use LitExpr::*;
        match self {
            Integer(int) => write!(f, "{}", int),
            Float(float) => write!(f, "{}", float),
            Str(string) => write!(f, "{}", string),
            Boolean(boolean) => write!(f, "{}", boolean),
            Nil => write!(f, "nil"),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum UnOp {
    Negate,
    Not,
}

impl UnOp {
    fn to_string(&self) -> &'static str {
        use UnOp::*;
        match self {
            Negate => "-",
            Not => "!",
        }
    }
}

impl From<TokenKind> for UnOp {
    fn from(kind: TokenKind) -> Self {
        use TokenKind::*;
        use UnOp::*;
        match kind {
            Bang => Not,
            Minus => Negate,
            kind => panic!("Token kind '{:?}' is not a unary operator", kind),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

impl BinOp {
    fn to_string(&self) -> &'static str {
        use BinOp::*;
        match self {
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            Rem => "%",
            Equal => "==",
            NotEqual => "!=",
            Greater => ">",
            GreaterEqual => ">=",
            Less => "<",
            LessEqual => "<=",
        }
    }
}

impl From<TokenKind> for BinOp {
    fn from(kind: TokenKind) -> Self {
        use TokenKind::*;
        match kind {
            Plus | PlusEqual | PlusPlus => BinOp::Add,
            Minus | MinusEqual | MinusMinus => BinOp::Sub,
            Star | StarEqual => BinOp::Mul,
            Slash | SlashEqual => BinOp::Div,
            Percent | PercentEqual => BinOp::Rem,
            EqualEqual => BinOp::Equal,
            BangEqual => BinOp::NotEqual,
            Greater => BinOp::Greater,
            GreaterEqual => BinOp::GreaterEqual,
            Less => BinOp::Less,
            LessEqual => BinOp::LessEqual,
            kind => panic!("Token kind '{:?}' is not a binary operator", kind),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum LogOp {
    And,
    Or,
}

impl LogOp {
    fn to_string(&self) -> &'static str {
        use LogOp::*;
        match self {
            And => "and",
            Or => "or",
        }
    }
}

impl From<TokenKind> for LogOp {
    fn from(kind: TokenKind) -> Self {
        use TokenKind::*;
        match kind {
            And => LogOp::And,
            Or => LogOp::Or,
            kind => panic!("Token kind '{:?}' is not a logical operator", kind),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    #[test]
    fn test_debug() {
        // (1 + 2) * (4 - 3)
        let expr = mult_expr(
            group_expr(
                add_expr(int_expr(1, (0, 1)), int_expr(2, (0, 5)), (0, 3)),
                (0, 0),
            ),
            group_expr(
                sub_expr(int_expr(4, (0, 11)), int_expr(3, (0, 15)), (0, 13)),
                (0, 10),
            ),
            (0, 8),
        );
        assert_eq!(
            format!("{:?}", expr),
            "(* (group (+ 1[1:1] 2[1:5])[1:3])[1:0] (group (- 4[1:11] 3[1:15])[1:13])[1:10])[1:8]"
        );
    }
}
