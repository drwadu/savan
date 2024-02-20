use clingo::Symbol;

/// Converts facet to [String](https://doc.rust-lang.org/std/string/struct.String.html).
pub fn repr(symbol: Symbol) -> String {
    symbol.to_string()
}

pub(crate) fn parse(exp: &str) -> Option<Symbol> {
    match clingo::parse_term(exp) {
        Ok(s) => Some(s),
        Err(e) => {
            println!("parsing {:?} failed with {:?}", exp, e);
            None
        }
    }
}
