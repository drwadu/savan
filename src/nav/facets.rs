use super::utils::ToHashSet;
use super::Navigator;
use clingo::{SolverLiteral, Symbol};
use std::collections::HashSet;

pub(crate) fn consequences(
    nav: &mut Navigator,
    route: &[SolverLiteral],
    kind: &str,
) -> Option<Vec<Symbol>> {
    let mut ctl = nav.ctl.take()?;
    ctl.configuration_mut()
        .map(|c| {
            c.root()
                .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                .map(|sk| c.value_set(sk, kind))
                .ok()
        })
        .ok()?;

    let mut xs = vec![];
    let mut handle = ctl.solve(clingo::SolveMode::YIELD, route).ok()?;

    while let Ok(Some(ys)) = handle.model() {
        xs = ys.symbols(clingo::ShowType::SHOWN).ok()?;
        handle.resume().ok()?;
    }
    let mut ctl = handle.close().ok()?;
    ctl.configuration_mut()
        .map(|c| {
            c.root()
                .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                .map(|sk| c.value_set(sk, "auto"))
                .ok()
        })
        .ok()?;
    nav.ctl = Some(ctl);

    Some(xs)
}

/// Functionalities revolving around facets of a program.
pub trait Facets {
    /// Returns brave consequences found under **route**.
    fn brave_consequences<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>>;
    /// Returns cautious consequences found under **route**.
    fn cautious_consequences<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>>;
    /// Returns facet-inducing atoms found under **route**.
    fn facet_inducing_atoms<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<HashSet<Symbol>>;
}
impl Facets for Navigator {
    fn brave_consequences<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>> {
        let route = peek_on
            .map(|s| self.expression_to_literal(s))
            .flatten()
            .collect::<Vec<_>>();
        consequences(self, &route, "brave")
    }

    fn cautious_consequences<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>> {
        let route = peek_on
            .map(|s| self.expression_to_literal(s))
            .flatten()
            .collect::<Vec<_>>();

        consequences(self, &route, "cautious")
    }

    fn facet_inducing_atoms<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<HashSet<Symbol>> {
        let route = peek_on
            .map(|s| self.expression_to_literal(s))
            .flatten()
            .collect::<Vec<_>>();

        let bcs = consequences(self, &route, "brave")?;

        match !bcs.is_empty() {
            true => consequences(self, &route, "cautious")
                .as_ref()
                .and_then(|ccs| Some(bcs.difference_as_set(ccs))),
            _ => Some(bcs.to_hashset()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::nav::errors::NavigatorError;

    use super::super::errors::Result;
    use super::super::lex;
    use super::*;

    #[test]
    fn brave() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;

        let bcs = nav
            .brave_consequences(["a"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(bcs.len(), 2);
        assert!(bcs.contains(&lex::parse("a").ok_or(NavigatorError::None)?));
        assert!(bcs.contains(&lex::parse("e").ok_or(NavigatorError::None)?));

        let bcs = nav
            .brave_consequences(["b"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(bcs.len(), 4);
        assert!(bcs.contains(&lex::parse("b").ok_or(NavigatorError::None)?));
        assert!(bcs.contains(&lex::parse("c").ok_or(NavigatorError::None)?));
        assert!(bcs.contains(&lex::parse("d").ok_or(NavigatorError::None)?));
        assert!(bcs.contains(&lex::parse("e").ok_or(NavigatorError::None)?));

        let bcs = nav
            .brave_consequences(["a", "b"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(bcs.len(), 0);

        Ok(())
    }

    #[test]
    fn cautious() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;

        let ccs = nav
            .cautious_consequences(["a"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(ccs.len(), 2);
        assert!(ccs.contains(&lex::parse("a").ok_or(NavigatorError::None)?));
        assert!(ccs.contains(&lex::parse("e").ok_or(NavigatorError::None)?));

        let ccs = nav
            .cautious_consequences(["b"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(ccs.len(), 2);
        assert!(ccs.contains(&lex::parse("b").ok_or(NavigatorError::None)?));
        assert!(ccs.contains(&lex::parse("e").ok_or(NavigatorError::None)?));

        let ccs = nav
            .cautious_consequences(["a", "b"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(ccs.len(), 0);

        Ok(())
    }

    #[test]
    fn facet_inducing() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;

        let fs = nav
            .facet_inducing_atoms(["a"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(fs.len(), 0);

        let fs = nav
            .facet_inducing_atoms(["b"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(fs.len(), 2);
        assert!(fs.contains(&lex::parse("c").ok_or(NavigatorError::None)?));
        assert!(fs.contains(&lex::parse("d").ok_or(NavigatorError::None)?));

        let fs = nav
            .facet_inducing_atoms(["a", "b"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(fs.len(), 0);

        let fs = nav
            .facet_inducing_atoms(["~c"].iter())
            .ok_or(NavigatorError::None)?;
        assert_eq!(fs.len(), 3);
        assert!(fs.contains(&lex::parse("a").ok_or(NavigatorError::None)?));
        assert!(fs.contains(&lex::parse("b").ok_or(NavigatorError::None)?));
        assert!(fs.contains(&lex::parse("d").ok_or(NavigatorError::None)?));

        Ok(())
    }
}
