use crate::clarity::representations::{SymbolicExpression, PreSymbolicExpression};
use crate::clarity::diagnostic::{Diagnostic, DiagnosableError};
use crate::clarity::types::{TypeSignature, TupleTypeSignature};
use crate::clarity::MAX_CALL_STACK_DEPTH;
use std::error;
use std::fmt;
use crate::clarity::costs::{ExecutionCost, CostErrors};

pub type ParseResult <T> = Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum ParseErrors {
    CostOverflow,
    CostBalanceExceeded(ExecutionCost, ExecutionCost),
    MemoryBalanceExceeded(u64, u64),
    TooManyExpressions,
    ExpressionStackDepthTooDeep,
    FailedCapturingInput,
    SeparatorExpected(String),
    SeparatorExpectedAfterColon(String),
    ProgramTooLarge,
    IllegalVariableName(String),
    IllegalContractName(String),
    UnknownQuotedValue(String),
    FailedParsingIntValue(String),
    FailedParsingBuffer(String),
    FailedParsingHexValue(String, String),
    FailedParsingPrincipal(String),
    FailedParsingField(String),
    FailedParsingRemainder(String),
    ClosingParenthesisUnexpected,
    ClosingParenthesisExpected,
    ClosingTupleLiteralUnexpected,
    ClosingTupleLiteralExpected,
    CircularReference(Vec<String>),
    TupleColonExpected(usize),
    TupleCommaExpected(usize),
    TupleItemExpected(usize),
    NameAlreadyUsed(String),
    TraitReferenceNotAllowed,
    ImportTraitBadSignature,
    DefineTraitBadSignature,
    ImplTraitBadSignature,
    TraitReferenceUnknown(String),
    CommaSeparatorUnexpected,
    ColonSeparatorUnexpected,
    InvalidCharactersDetected,
    InvalidEscaping,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub err: ParseErrors,
    pub pre_expressions: Option<Vec<PreSymbolicExpression>>,
    pub diagnostic: Diagnostic,
}

impl ParseError {
    pub fn new(err: ParseErrors) -> ParseError {
        let diagnostic = Diagnostic::err(&err);
        ParseError {
            err,
            pre_expressions: None,
            diagnostic
        }
    }

    pub fn has_pre_expression(&self) -> bool {
        self.pre_expressions.is_some()
    }

    pub fn set_pre_expression(&mut self, expr: &PreSymbolicExpression) {
        self.diagnostic.spans = vec![expr.span.clone()];
        self.pre_expressions.replace(vec![expr.clone()]);
    }

    pub fn set_pre_expressions(&mut self, exprs: Vec<PreSymbolicExpression>) {
        self.diagnostic.spans = exprs.iter().map(|e| e.span.clone()).collect();
        self.pre_expressions.replace(exprs.clone().to_vec());
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.err {
            _ =>  write!(f, "{:?}", self.err)
        }?;

        if let Some(ref e) = self.pre_expressions {
            write!(f, "\nNear:\n{:?}", e)?;
        }

        Ok(())
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.err {
            _ => None
        }
    }
}

impl From<ParseErrors> for ParseError {
    fn from(err: ParseErrors) -> Self {
        ParseError::new(err)
    }
}


impl From<CostErrors> for ParseError {
    fn from(err: CostErrors) -> Self {
        match err {
            CostErrors::CostOverflow => ParseError::new(ParseErrors::CostOverflow),
            CostErrors::CostBalanceExceeded(a,b) => ParseError::new(ParseErrors::CostBalanceExceeded(a,b)),
            CostErrors::MemoryBalanceExceeded(a,b) => ParseError::new(ParseErrors::MemoryBalanceExceeded(a,b)),
        }
    }
}

impl DiagnosableError for ParseErrors {

    fn message(&self) -> String {
        match &self {
            ParseErrors::CostOverflow => format!("used up cost budget during the parse"),
            ParseErrors::CostBalanceExceeded(bal, used) => format!("used up cost budget during the parse: {} balance, {} used", bal, used),
            ParseErrors::MemoryBalanceExceeded(bal, used) => format!("used up memory budget during the parse: {} balance, {} used", bal, used),
            ParseErrors::TooManyExpressions => format!("too many expressions"),
            ParseErrors::FailedCapturingInput => format!("failed to capture value from input"),
            ParseErrors::SeparatorExpected(found) => format!("expected whitespace or a close parens. Found: '{}'", found),
            ParseErrors::SeparatorExpectedAfterColon(found) => format!("whitespace expected after colon (:), Found: '{}'", found),
            ParseErrors::ProgramTooLarge => format!("program too large to parse"),
            ParseErrors::IllegalContractName(contract_name) => format!("illegal contract name: '{}'", contract_name),
            ParseErrors::IllegalVariableName(var_name) => format!("illegal variable name: '{}'", var_name),
            ParseErrors::UnknownQuotedValue(value) => format!("unknown 'quoted value '{}'", value),
            ParseErrors::FailedParsingIntValue(value) => format!("failed to parse int literal '{}'", value),
            ParseErrors::FailedParsingHexValue(value, x) => format!("invalid hex-string literal {}: {}", value, x),
            ParseErrors::FailedParsingPrincipal(value) => format!("invalid principal literal: {}", value),
            ParseErrors::FailedParsingBuffer(value) => format!("invalid buffer literal: {}", value),
            ParseErrors::FailedParsingField(value) => format!("invalid field literal: {}", value),
            ParseErrors::FailedParsingRemainder(remainder) => format!("failed to lex input remainder: '{}'", remainder),
            ParseErrors::ClosingParenthesisUnexpected => format!("tried to close list which isn't open"),
            ParseErrors::ClosingParenthesisExpected => format!("list expressions (..) left opened"),
            ParseErrors::ClosingTupleLiteralUnexpected => format!("tried to close tuple literal which isn't open"),
            ParseErrors::ClosingTupleLiteralExpected => format!("tuple literal {{..}} left opened"),
            ParseErrors::ColonSeparatorUnexpected => format!("misplaced colon"),
            ParseErrors::CommaSeparatorUnexpected => format!("misplaced comma"),
            ParseErrors::TupleColonExpected(i) => format!("tuple literal construction expects a colon at index {}", i),
            ParseErrors::TupleCommaExpected(i) => format!("tuple literal construction expects a comma at index {}", i),
            ParseErrors::TupleItemExpected(i) => format!("tuple literal construction expects a key or value at index {}", i),
            ParseErrors::CircularReference(function_names) => format!("detected interdependent functions ({})", function_names.join(", ")),
            ParseErrors::NameAlreadyUsed(name) => format!("defining '{}' conflicts with previous value", name),
            ParseErrors::ImportTraitBadSignature => format!("(use-trait ...) expects a trait name and a trait identifier"),
            ParseErrors::DefineTraitBadSignature => format!("(define-trait ...) expects a trait name and a trait definition"),
            ParseErrors::ImplTraitBadSignature => format!("(impl-trait ...) expects a trait identifier"),
            ParseErrors::TraitReferenceNotAllowed => format!("trait references can not be stored"),
            ParseErrors::TraitReferenceUnknown(trait_name) => format!("use of undeclared trait <{}>", trait_name),
            ParseErrors::ExpressionStackDepthTooDeep => format!("AST has too deep of an expression nesting. The maximum stack depth is {}", MAX_CALL_STACK_DEPTH),
            ParseErrors::InvalidCharactersDetected => format!("invalid characters detected"),
            ParseErrors::InvalidEscaping => format!("invalid escaping detected in string"),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match &self {
            _ => None
        }
    }
}
