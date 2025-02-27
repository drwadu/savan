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

pub(crate) fn consequences_projecting(
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

    ctl.configuration_mut()
        .map(|c| {
            c.root()
                .and_then(|rk| c.map_at(rk, "solve.project"))
                .map(|sk| c.value_set(sk, "show"))
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
    ctl.configuration_mut()
        .map(|c| {
            c.root()
                .and_then(|rk| c.map_at(rk, "solve.project"))
                .map(|sk| c.value_set(sk, "auto"))
                .ok()
        })
        .ok()?;
    nav.ctl = Some(ctl);

    Some(xs)
}

pub(crate) fn consequences_count(
    nav: &mut Navigator,
    route: &[SolverLiteral],
    kind: &str,
) -> Option<usize> {
    let mut ctl = nav.ctl.take()?;
    ctl.configuration_mut()
        .map(|c| {
            c.root()
                .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                .map(|sk| c.value_set(sk, kind))
                .ok()
        })
        .ok()?;

    let mut count = 0;
    let mut handle = ctl.solve(clingo::SolveMode::YIELD, route).ok()?;

    while let Ok(Some(ys)) = handle.model() {
        count = ys.symbols(clingo::ShowType::SHOWN).ok()?.len();
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

    Some(count)
}

pub(crate) fn consequences_count_projecting(
    nav: &mut Navigator,
    route: &[SolverLiteral],
    kind: &str,
) -> Option<usize> {
    let mut ctl = nav.ctl.take()?;
    ctl.configuration_mut()
        .map(|c| {
            c.root()
                .and_then(|rk| c.map_at(rk, "solve.enum_mode"))
                .map(|sk| c.value_set(sk, kind))
                .ok()
        })
        .ok()?;

    ctl.configuration_mut()
        .map(|c| {
            c.root()
                .and_then(|rk| c.map_at(rk, "solve.project"))
                .map(|sk| c.value_set(sk, "show"))
                .ok()
        })
        .ok()?;


    let mut count = 0;
    let mut handle = ctl.solve(clingo::SolveMode::YIELD, route).ok()?;

    while let Ok(Some(ys)) = handle.model() {
        count = ys.symbols(clingo::ShowType::SHOWN).ok()?.len();
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
    ctl.configuration_mut()
        .map(|c| {
            c.root()
                .and_then(|rk| c.map_at(rk, "solve.project"))
                .map(|sk| c.value_set(sk, "auto"))
                .ok()
        })
        .ok()?;
    nav.ctl = Some(ctl);

    Some(count)
}

/// Functionalities revolving around facets of a program.
pub trait Facets {
    /// Colors for truth values.
    const T: &'static str = "\x1b[0;30;42m[T]\x1b[0m";
    const U: &'static str = "\x1b[0;30;44m[U]\x1b[0m";
    const F: &'static str = "\x1b[0;30;41m[F]\x1b[0m";
    /// Returns brave consequences found under **route**.
    fn brave_consequences<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>>;
    /// Returns brave consequences found under **route**, while projecting onto shown atoms.
    fn brave_consequences_projecting<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>>;
    /// Returns cautious consequences found under **route**.
    fn cautious_consequences<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>>;
    /// Returns cautious consequences found under **route**, while projecting onto shown atoms.
    fn cautious_consequences_projecting<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>>;
    /// Returns facet-inducing atoms found under **route**.
    fn facet_inducing_atoms<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<HashSet<Symbol>>;
    /// Returns facet-inducing atoms found under **route**, while projecting onto shown atoms.
    fn facet_inducing_atoms_projecting<S: ToString>(
        &mut self,
        route: impl Iterator<Item = S>,
    ) -> Option<HashSet<Symbol>>;
    /// Prints literals modeled under **route**, and returns facet-inducing atoms under **route**.
    fn learned_that(
        &mut self,
        facets: &[String],
        route: &[String],
        write_to: Option<String>,
    ) -> Option<Vec<String>>;
    /// Prints literals modeled under **route**, and returns facet-inducing atoms under **route**, while projecting onto shown atoms.
    fn learned_that_projecting(
        &mut self,
        facets: &[String],
        route: &[String],
        write_to: Option<String>,
    ) -> Option<Vec<String>>;
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

    fn brave_consequences_projecting<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>> {
        let route = peek_on
            .map(|s| self.expression_to_literal(s))
            .flatten()
            .collect::<Vec<_>>();

        consequences_projecting(self, &route, "brave")
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

    fn cautious_consequences_projecting<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<Vec<Symbol>> {
        let route = peek_on
            .map(|s| self.expression_to_literal(s))
            .flatten()
            .collect::<Vec<_>>();

        consequences_projecting(self, &route, "cautious")
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

    fn facet_inducing_atoms_projecting<S: ToString>(
        &mut self,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<HashSet<Symbol>> {
        let route = peek_on
            .map(|s| self.expression_to_literal(s))
            .flatten()
            .collect::<Vec<_>>();

        let bcs = consequences_projecting(self, &route, "brave")?;

        match !bcs.is_empty() {
            true => consequences_projecting(self, &route, "cautious")
                .as_ref()
                .and_then(|ccs| Some(bcs.difference_as_set(ccs))),
            _ => Some(bcs.to_hashset()),
        }
    }

    fn learned_that(
        &mut self,
        facets: &[String],
        route: &[String],
        write_to: Option<String>,
    ) -> Option<Vec<String>> {
        let bc = self
            .brave_consequences(route.iter())
            .map(|xs| xs.iter().map(|f| f.to_string()).collect::<Vec<_>>())?;
        let cc = self
            .cautious_consequences(route.iter())
            .map(|xs| xs.iter().map(|f| f.to_string()).collect::<Vec<_>>())?;

        if let Some(_path) = write_to {
            todo!()
        }
        {
            facets.into_iter().for_each(|f| match cc.contains(&f) {
                true => println!("{f}"),
                _ => match !bc.contains(&f) {
                    true => println!("~{f}"),
                    _ => (),
                },
            });
        }

        match !bc.is_empty() {
            true => Some(
                bc.difference_as_set(&cc)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
            _ => Some(bc),
        }
    }

    fn learned_that_projecting(
        &mut self,
        facets: &[String],
        route: &[String],
        write_to: Option<String>,
    ) -> Option<Vec<String>> {
        let bc = self
            .brave_consequences_projecting(route.iter())
            .map(|xs| xs.iter().map(|f| f.to_string()).collect::<Vec<_>>())?;
        let cc = self
            .cautious_consequences_projecting(route.iter())
            .map(|xs| xs.iter().map(|f| f.to_string()).collect::<Vec<_>>())?;

        if let Some(_path) = write_to {
            todo!()
        }
        {
            facets.into_iter().for_each(|f| match cc.contains(&f) {
                true => println!("{f}"),
                _ => match !bc.contains(&f) {
                    true => println!("~{f}"),
                    _ => (),
                },
            });
        }

        match !bc.is_empty() {
            true => Some(
                bc.difference_as_set(&cc)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
            _ => Some(bc),
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

    #[test]
    fn learned_that() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;

        let fs = nav
            .learned_that(
                &vec![
                    "a".to_owned(),
                    "b".to_owned(),
                    "c".to_owned(),
                    "d".to_owned(),
                ],
                &vec!["a".to_owned()],
                None,
            )
            .ok_or(NavigatorError::None)?;
        assert_eq!(fs.len(), 0);

        Ok(())
    }
}
