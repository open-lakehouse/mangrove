// Vendored from unitycatalog-rs (github.com/ion-elgreco/unitycatalog-rs,
// crates/uc-server/src/routes/control/scim_filter.rs), Apache-2.0.

//! SCIM filters (RFC 7644 §3.4.2.2), as upstream parses and evaluates them
//! with the UnboundID SDK: `and`, `or`, `not(…)`, grouping, value paths
//! (`emails[type eq "work"]`) and the operators `eq ne co sw ew gt ge lt le
//! pr`. Attribute names, keywords and string comparisons ignore case;
//! dates compare as instants.

use serde_json::Value;

/// A parsed filter.
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    And(Box<Filter>, Box<Filter>),
    Or(Box<Filter>, Box<Filter>),
    Not(Box<Filter>),
    Present(AttrPath),
    Compare(AttrPath, Op, Value),
    /// `attr[filter]`: some value of `attr` matches `filter`.
    ValuePath(AttrPath, Box<Filter>),
}

/// `attr` or `attr.sub`, after any schema URN prefix.
#[derive(Debug, Clone, PartialEq)]
pub struct AttrPath {
    attr: String,
    sub: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Eq,
    Ne,
    Co,
    Sw,
    Ew,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Open,
    Close,
    OpenBracket,
    CloseBracket,
    Word(String),
    Literal(Value),
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            c if c.is_whitespace() => i += 1,
            '(' => {
                tokens.push(Token::Open);
                i += 1;
            }
            ')' => {
                tokens.push(Token::Close);
                i += 1;
            }
            '[' => {
                tokens.push(Token::OpenBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::CloseBracket);
                i += 1;
            }
            '"' => {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    i += if chars[i] == '\\' { 2 } else { 1 };
                }
                if i >= chars.len() {
                    return Err(format!("Unterminated string starting at position {start}"));
                }
                i += 1;
                let quoted: String = chars[start..i].iter().collect();
                let value: String = serde_json::from_str(&quoted)
                    .map_err(|_| format!("Invalid string {quoted}"))?;
                tokens.push(Token::Literal(Value::String(value)));
            }
            _ => {
                let start = i;
                while i < chars.len() && !chars[i].is_whitespace() && !"()[]\"".contains(chars[i]) {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let first = word.chars().next().unwrap_or_default();
                let number = (first.is_ascii_digit() || first == '-')
                    .then(|| serde_json::from_str::<serde_json::Number>(&word).ok())
                    .flatten();
                tokens.push(match number {
                    Some(n) => Token::Literal(Value::Number(n)),
                    None => Token::Word(word),
                });
            }
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.at).cloned();
        self.at += 1;
        token
    }

    fn keyword(&self, word: &str) -> bool {
        matches!(self.peek(), Some(Token::Word(w)) if w.eq_ignore_ascii_case(word))
    }

    fn expect(&mut self, token: Token, what: &str) -> Result<(), String> {
        match self.next() {
            Some(t) if t == token => Ok(()),
            Some(t) => Err(format!("Expected {what} but found {}", describe(&t))),
            None => Err(format!("Expected {what} but reached the end of the filter")),
        }
    }

    fn or(&mut self) -> Result<Filter, String> {
        let mut filter = self.and()?;
        while self.keyword("or") {
            self.at += 1;
            filter = Filter::Or(Box::new(filter), Box::new(self.and()?));
        }
        Ok(filter)
    }

    fn and(&mut self) -> Result<Filter, String> {
        let mut filter = self.unary()?;
        while self.keyword("and") {
            self.at += 1;
            filter = Filter::And(Box::new(filter), Box::new(self.unary()?));
        }
        Ok(filter)
    }

    fn unary(&mut self) -> Result<Filter, String> {
        if self.keyword("not") && self.tokens.get(self.at + 1) == Some(&Token::Open) {
            self.at += 2;
            let inner = self.or()?;
            self.expect(Token::Close, "')'")?;
            return Ok(Filter::Not(Box::new(inner)));
        }
        if self.peek() == Some(&Token::Open) {
            self.at += 1;
            let inner = self.or()?;
            self.expect(Token::Close, "')'")?;
            return Ok(inner);
        }
        let path = match self.next() {
            Some(Token::Word(word)) => attr_path(&word)?,
            Some(t) => {
                return Err(format!(
                    "Expected an attribute path but found {}",
                    describe(&t)
                ));
            }
            None => return Err("Unexpected end of filter string".to_string()),
        };
        if self.peek() == Some(&Token::OpenBracket) {
            self.at += 1;
            let inner = self.or()?;
            self.expect(Token::CloseBracket, "']'")?;
            return Ok(Filter::ValuePath(path, Box::new(inner)));
        }
        let op = match self.next() {
            Some(Token::Word(op)) => op.to_ascii_lowercase(),
            Some(t) => return Err(format!("Expected an operator but found {}", describe(&t))),
            None => return Err("Unexpected end of filter string".to_string()),
        };
        let op = match op.as_str() {
            "pr" => return Ok(Filter::Present(path)),
            "eq" => Op::Eq,
            "ne" => Op::Ne,
            "co" => Op::Co,
            "sw" => Op::Sw,
            "ew" => Op::Ew,
            "gt" => Op::Gt,
            "ge" => Op::Ge,
            "lt" => Op::Lt,
            "le" => Op::Le,
            other => return Err(format!("Unrecognized operator '{other}'")),
        };
        let value = match self.next() {
            Some(Token::Literal(value)) => value,
            Some(Token::Word(w)) if w.eq_ignore_ascii_case("true") => Value::Bool(true),
            Some(Token::Word(w)) if w.eq_ignore_ascii_case("false") => Value::Bool(false),
            Some(Token::Word(w)) if w.eq_ignore_ascii_case("null") => Value::Null,
            Some(t) => return Err(format!("Invalid comparison value {}", describe(&t))),
            None => return Err("Unexpected end of filter string".to_string()),
        };
        Ok(Filter::Compare(path, op, value))
    }
}

fn describe(token: &Token) -> String {
    match token {
        Token::Open => "'('".to_string(),
        Token::Close => "')'".to_string(),
        Token::OpenBracket => "'['".to_string(),
        Token::CloseBracket => "']'".to_string(),
        Token::Word(w) => format!("'{w}'"),
        Token::Literal(v) => v.to_string(),
    }
}

/// `[urn:…:]attr[.sub]`.
fn attr_path(word: &str) -> Result<AttrPath, String> {
    let name = word.rsplit_once(':').map_or(word, |(_, name)| name);
    let (attr, sub) = match name.split_once('.') {
        Some((attr, sub)) => (attr, Some(sub.to_string())),
        None => (name, None),
    };
    let valid = |s: &str| {
        s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '$')
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '$')
    };
    if !valid(attr) || sub.as_deref().is_some_and(|s| !valid(s)) {
        return Err(format!("Invalid attribute path '{word}'"));
    }
    Ok(AttrPath {
        attr: attr.to_string(),
        sub,
    })
}

/// Parse `input`; the error is the SCIM `invalidFilter` detail.
pub fn parse(input: &str) -> Result<Filter, String> {
    let mut parser = Parser {
        tokens: tokenize(input)?,
        at: 0,
    };
    let filter = parser.or()?;
    match parser.next() {
        None => Ok(filter),
        Some(t) => Err(format!("Unexpected {} after the filter", describe(&t))),
    }
}

/// The member `name` of `object`, ignoring case.
fn member<'a>(object: &'a Value, name: &str) -> Option<&'a Value> {
    object
        .as_object()?
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, v)| v)
}

/// The values at `path`, arrays flattened.
fn values<'a>(resource: &'a Value, path: &AttrPath) -> Vec<&'a Value> {
    let flatten = |v: &'a Value| -> Vec<&'a Value> {
        match v {
            Value::Array(items) => items.iter().collect(),
            other => vec![other],
        }
    };
    let Some(top) = member(resource, &path.attr) else {
        return vec![];
    };
    match &path.sub {
        None => flatten(top),
        Some(sub) => flatten(top)
            .into_iter()
            .filter_map(|v| member(v, sub))
            .flat_map(flatten)
            .collect(),
    }
}

fn present(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
        _ => true,
    }
}

fn instant(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    chrono::DateTime::parse_from_rfc3339(s).ok()
}

/// One value against the filter's: strings ignore case, dates compare as
/// instants; other kinds only to their own kind.
fn compare(actual: &Value, op: Op, expected: &Value) -> bool {
    use std::cmp::Ordering;
    let ordered = |ord: Ordering| match op {
        Op::Eq => ord == Ordering::Equal,
        Op::Ne => ord != Ordering::Equal,
        Op::Gt => ord == Ordering::Greater,
        Op::Ge => ord != Ordering::Less,
        Op::Lt => ord == Ordering::Less,
        Op::Le => ord != Ordering::Greater,
        Op::Co | Op::Sw | Op::Ew => false,
    };
    match (actual, expected) {
        (Value::String(a), Value::String(e)) => {
            if let (Some(a), Some(e)) = (instant(a), instant(e)) {
                return ordered(a.cmp(&e));
            }
            let (a, e) = (a.to_lowercase(), e.to_lowercase());
            match op {
                Op::Co => a.contains(&e),
                Op::Sw => a.starts_with(&e),
                Op::Ew => a.ends_with(&e),
                _ => ordered(a.cmp(&e)),
            }
        }
        (Value::Number(a), Value::Number(e)) => match (a.as_f64(), e.as_f64()) {
            (Some(a), Some(e)) => a.partial_cmp(&e).is_some_and(ordered),
            _ => false,
        },
        (Value::Bool(a), Value::Bool(e)) => matches!(op, Op::Eq | Op::Ne) && ordered(a.cmp(e)),
        _ => false,
    }
}

/// Whether `resource` (a SCIM resource as JSON) matches `filter`.
pub fn matches(filter: &Filter, resource: &Value) -> bool {
    match filter {
        Filter::And(a, b) => matches(a, resource) && matches(b, resource),
        Filter::Or(a, b) => matches(a, resource) || matches(b, resource),
        Filter::Not(inner) => !matches(inner, resource),
        Filter::Present(path) => values(resource, path).into_iter().any(present),
        Filter::Compare(path, op, Value::Null) => {
            let any = values(resource, path).into_iter().any(|v| !v.is_null());
            match op {
                Op::Eq => !any,
                Op::Ne => any,
                _ => false,
            }
        }
        // Not equal: no value is equal.
        Filter::Compare(path, Op::Ne, expected) => !values(resource, path)
            .into_iter()
            .any(|v| compare(v, Op::Eq, expected)),
        Filter::Compare(path, op, expected) => values(resource, path)
            .into_iter()
            .any(|v| compare(v, *op, expected)),
        Filter::ValuePath(path, inner) => match member(resource, &path.attr) {
            Some(Value::Array(items)) => items.iter().any(|item| matches(inner, item)),
            Some(item @ Value::Object(_)) => matches(inner, item),
            _ => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn user() -> Value {
        json!({
            "id": "2819c223",
            "userName": "Bjensen@example.com",
            "displayName": "Babs Jensen",
            "active": true,
            "emails": [
                { "value": "bjensen@example.com", "primary": true, "type": "work" },
                { "value": "babs@jensen.org", "type": "home" }
            ],
            "meta": { "resourceType": "User", "created": "2011-08-01T18:29:49.793Z" }
        })
    }

    fn check(filter: &str) -> bool {
        matches(
            &parse(filter).unwrap_or_else(|e| panic!("{filter}: {e}")),
            &user(),
        )
    }

    #[test]
    fn evaluates_upstream_filters() {
        for (filter, expected) in [
            (r#"userName eq "bjensen@example.com""#, true),
            (r#"USERNAME Eq "BJENSEN@EXAMPLE.COM""#, true),
            (
                r#"urn:ietf:params:scim:schemas:core:2.0:User:userName sw "bjen""#,
                true,
            ),
            (r#"displayName co "jens""#, true),
            (r#"displayName ew "sen""#, true),
            (r#"displayName ne "Babs Jensen""#, false),
            ("active eq true", true),
            ("active eq false", false),
            ("externalId pr", false),
            ("externalId eq null", true),
            ("displayName pr", true),
            (r#"emails.value eq "babs@jensen.org""#, true),
            (r#"emails[type eq "work" and value co "example.com"]"#, true),
            (
                r#"emails[type eq "home" and value co "example.com"]"#,
                false,
            ),
            (r#"meta.created gt "2011-01-01T00:00:00Z""#, true),
            (r#"meta.created lt "2011-08-01T18:29:49.793Z""#, false),
            (r#"meta.created ge "2011-08-01T18:29:49.793+00:00""#, true),
            (r#"userName eq "x" or displayName sw "babs""#, true),
            (
                r#"userName eq "x" or displayName sw "babs" and active eq false"#,
                false,
            ),
            (
                r#"(userName eq "x" or displayName sw "babs") and active eq true"#,
                true,
            ),
            (r#"not (userName eq "x")"#, true),
            (r#"not(active eq true)"#, false),
        ] {
            assert_eq!(check(filter), expected, "{filter}");
        }
    }

    #[test]
    fn refuses_malformed_filters() {
        for filter in [
            "",
            "userName",
            r#"userName xx "a""#,
            r#"userName eq"#,
            r#"(userName eq "a""#,
            r#"userName eq "a" junk"#,
            r#"userName eq "unterminated"#,
            r#"emails[type eq "work""#,
        ] {
            assert!(parse(filter).is_err(), "{filter}");
        }
    }
}
