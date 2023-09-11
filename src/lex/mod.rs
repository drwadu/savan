use clingo::Symbol;

pub(crate) fn parse(exp: &str) -> Option<Symbol> {
    match clingo::parse_term(exp) {
        Ok(s) => Some(s),
        Err(e) => {
            println!("parsing {:?} failed with {:?}", exp, e);
            None
        }
    }
}

