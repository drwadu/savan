pub mod errors;
pub mod facets;
pub mod soe;
mod utils;
pub mod weights;

use crate::lex;

use errors::Result;

use clingo::{Control, Part, SolverLiteral, Symbol};
use std::collections::HashMap;

use self::errors::NavigatorError;

pub struct Navigator {
    source: (String, Vec<String>),
    ctl: Option<Control>,
    literals: HashMap<Symbol, SolverLiteral>,
}
impl Navigator {
    /// Constructs [Navigator](Navigator) over answer set program specified by **source**.
    ///
    /// The underlying clingo solver uses arguments specified in **args**.
    pub fn new(source: impl Into<String>, args: Vec<String>) -> Result<Self> {
        let mut ctl = clingo::control(args.clone())?;

        let lp = source.into();
        ctl.add("base", &[], &lp)?;
        ctl.ground(&[Part::new("base", vec![])?])?;

        let mut literals = HashMap::new();
        for atom in ctl.symbolic_atoms()?.iter()? {
            literals.insert(atom.symbol()?, atom.literal()?);
        }

        Ok(Self {
            source: (lp, args),
            ctl: Some(ctl),
            literals,
        })
    }

    /// Enumerates solutions under current route extended by facets in **route**.
    ///
    /// Will enumerate all existing solutions, if **upper_bound** is
    /// [None](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None).
    /// Otherwise, enumeration stops after **upper_bound** was reached.
    ///
    /// Prints the solutions, and returns the number of enumerated solutions.
    pub fn enumerate_solutions<S: ToString>(
        &mut self,
        upper_bound: Option<usize>,
        route: impl Iterator<Item = S>,
    ) -> Result<usize> {
        let ctl = self.ctl.take().ok_or(NavigatorError::NoControl)?;
        let ctx = route.map(|s| self.expression_to_literal(s)).flatten();
        let mut handle = ctl.solve(clingo::SolveMode::YIELD, &ctx.collect::<Vec<_>>())?;
        let mut i = 0;

        match upper_bound {
            None => {
                while let Ok(Some(answer_set)) = handle.model() {
                    println!("solution {:?}: ", i + 1);
                    let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                    atoms.iter().for_each(|atom| {
                        print!("{} ", atom.to_string());
                    });
                    println!();

                    i += 1;
                    handle.resume()?;
                }
            }
            Some(n) => {
                while let Ok(Some(answer_set)) = handle.model() {
                    println!("solution {:?}: ", i + 1);
                    let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                    atoms.iter().for_each(|atom| {
                        print!("{} ", atom.to_string());
                    });
                    println!();

                    i += 1;
                    if i == n {
                        break;
                    }
                    handle.resume()?;
                }
            }
        }

        let ctl = handle
            .close()
            .map_err(|e| errors::NavigatorError::Clingo(e))?;
        self.ctl = Some(ctl);

        return Ok(i);
    }

    /// Enumerates solutions under current route extended by facets in **route** and projected onto
    /// **project_on**.
    ///
    /// Will enumerate all existing solutions, if **upper_bound** is
    /// [None](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None).
    /// Otherwise, enumeration stops after **upper_bound** was reached.
    ///
    /// Prints the solutions, and returns the number of enumerated solutions.
    pub fn enumerate_projected_solutions<S: ToString>(
        &mut self,
        upper_bound: Option<usize>,
        route: impl Iterator<Item = S>,
        project_on: Vec<String>,
    ) -> Result<usize> {
        let ctl = self.ctl.take().ok_or(NavigatorError::NoControl)?;
        let ctx = route.map(|s| self.expression_to_literal(s)).flatten();
        let mut handle = ctl.solve(clingo::SolveMode::YIELD, &ctx.collect::<Vec<_>>())?;
        let mut i = 0;

        match upper_bound {
            None => {
                while let Ok(Some(answer_set)) = handle.model() {
                    println!("solution {:?}: ", i + 1);
                    let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                    atoms
                        .iter()
                        .map(|atom| atom.to_string())
                        .filter(|atom| project_on.contains(atom))
                        .for_each(|atom| {
                            print!("{} ", atom);
                        });
                    println!();

                    i += 1;
                    handle.resume()?;
                }
            }
            Some(n) => {
                while let Ok(Some(answer_set)) = handle.model() {
                    println!("solution {:?}: ", i + 1);
                    let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                    atoms
                        .iter()
                        .map(|atom| atom.to_string())
                        .filter(|atom| project_on.contains(atom))
                        .for_each(|atom| {
                            print!("{} ", atom);
                        });
                    println!();

                    i += 1;
                    if i == n {
                        break;
                    }
                    handle.resume()?;
                }
            }
        }

        let ctl = handle
            .close()
            .map_err(|e| errors::NavigatorError::Clingo(e))?;
        self.ctl = Some(ctl);

        return Ok(i);
    }

    /// Enumerates solutions under current route extended by facets in **route**
    /// in format required by clingraph.
    ///
    /// Will enumerate all existing solutions, if **upper_bound** is
    /// [None](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None).
    /// Otherwise, enumeration stops after **upper_bound** was reached.
    ///
    /// Returns one JSON per solution.
    #[allow(unused_assignments)]
    pub fn enumerate_solutions_outf2<S: ToString>(
        &mut self,
        upper_bound: Option<usize>,
        route: impl Iterator<Item = S>,
    ) -> Result<Vec<String>> {
        let ctl = self.ctl.take().ok_or(NavigatorError::NoControl)?;
        let ctx = route.map(|s| self.expression_to_literal(s)).flatten();
        let mut handle = ctl.solve(clingo::SolveMode::YIELD, &ctx.collect::<Vec<_>>())?;
        let mut i = 0;

        let mut out = vec![];

        match upper_bound {
            None => {
                while let Ok(Some(answer_set)) = handle.model() {
                    print!(".");
                    let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                    let mut answer_set = "{\"Solver\": \"\", \"Input\": [\"\"], ".to_owned();
                    answer_set =
                        format!("{answer_set}\"Call\": [ {{ \"Witnesses\": [ {{ \"Value\": [");
                    if let Some((last, rest)) = atoms.split_last() {
                        for atom in rest {
                            answer_set = format!("{answer_set}{:?}, ", atom.to_string());
                        }
                        answer_set = format!(
                            "{answer_set}{:?}]}}]}}],\n\"Result\":\"SATISFIABLE\",{}{}{}",
                            last.to_string(),
                            "\n\"Models\":{\"Number\":1,\"More\":\"yes\"},\n\"Calls\": 1,\n",
                            "\"Time\":{\"Total\": 0.000,\"Solve\": 0.000,",
                            "\"Model\": 0.000,\"Unsat\": 0.000,\"CPU\": 0.000}}\n"
                        );
                    }

                    i += 1;
                    out.push(answer_set);
                    handle.resume()?;
                }
            }
            Some(n) => {
                while let Ok(Some(answer_set)) = handle.model() {
                    print!(".");
                    let atoms = answer_set.symbols(clingo::ShowType::SHOWN)?;
                    let mut answer_set = "{\"Solver\": \"\", \"Input\": [\"\"], ".to_owned();
                    answer_set =
                        format!("{answer_set}\"Call\": [ {{ \"Witnesses\": [ {{ \"Value\": [");
                    if let Some((last, rest)) = atoms.split_last() {
                        for atom in rest {
                            answer_set = format!("{answer_set}{:?}, ", atom.to_string());
                        }
                        answer_set = format!(
                            "{answer_set}{:?}]}}]}}],\n\"Result\":\"SATISFIABLE\",{}{}{}",
                            last.to_string(),
                            "\n\"Models\":{\"Number\":1,\"More\":\"yes\"},\n\"Calls\": 1,\n",
                            "\"Time\":{\"Total\": 0.000,\"Solve\": 0.000,",
                            "\"Model\": 0.000,\"Unsat\": 0.000,\"CPU\": 0.000}}\n"
                        );
                    }

                    i += 1;
                    out.push(answer_set);
                    if i == n {
                        break;
                    }
                    handle.resume()?;
                }
            }
        }

        let ctl = handle
            .close()
            .map_err(|e| errors::NavigatorError::Clingo(e))?;
        self.ctl = Some(ctl);

        return Ok(out);
    }

    /// Enumerates solutions under current route extended by facets in **route**, quietly.
    ///
    /// Will enumerate all existing solutions, if **upper_bound** is
    /// [None](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None).
    /// Otherwise, enumeration stops after **upper_bound** was reached.
    ///
    /// Returns the number of enumerated solutions.
    pub fn enumerate_solutions_quietly<S: ToString>(
        &mut self,
        upper_bound: Option<usize>,
        route: impl Iterator<Item = S>,
    ) -> Result<usize> {
        let ctl = self.ctl.take().ok_or(NavigatorError::NoControl)?;
        let ctx = route.map(|s| self.expression_to_literal(s)).flatten();
        let mut handle = ctl.solve(clingo::SolveMode::YIELD, &ctx.collect::<Vec<_>>())?;

        let mut i = 0;

        match upper_bound {
            None => {
                while let Ok(Some(_)) = handle.model() {
                    i += 1;
                    handle.resume()?;
                }
            }
            Some(n) => {
                while let Ok(Some(_)) = handle.model() {
                    i += 1;
                    if i == n {
                        break;
                    }
                    handle.resume()?;
                }
            }
        }

        let ctl = handle
            .close()
            .map_err(|e| errors::NavigatorError::Clingo(e))?;
        self.ctl = Some(ctl);

        return Ok(i);
    }

    /// Checks whether **atom** is part of herbrand base.
    pub fn is_known(&self, atom: String) -> Option<bool> {
        lex::parse(&atom).map(|x| self.literals.keys().any(|y| *y == x))
    }

    /// Returns atoms of ground program.
    pub fn atoms(&self) -> impl Iterator<Item = String> + '_ {
        self.literals.keys().map(|sym| sym.to_string())
    }

    /// Returns atoms of ground program.
    pub fn symbols(&self) -> impl Iterator<Item = (String,usize)> + '_ {
        self.literals.keys().map(|s| (s.name().unwrap().to_owned(),s.arguments().unwrap().len()))
    }

    /// Adds specified `rule` from logic program.
    pub fn add_rule<S: std::fmt::Display>(&mut self, rule: S) -> Result<()> {
        let (source, args) = &self.source;
        let new_source = format!("{}\n{rule}", source);
        *self = Navigator::new(new_source, args.to_vec())?;

        Ok(())
    }

    /// Removes specified `rule` from logic program.
    pub fn remove_rule<S: ToString>(&mut self, rule: S) -> Result<()> {
        let (source, args) = &self.source;
        let new_source = source.replace(&rule.to_string(), "");
        *self = Navigator::new(new_source, args.to_vec())?;

        Ok(())
    }

    /// Adds specified `argument`.
    pub fn add_arg<S: std::fmt::Display>(&mut self, arg: S) -> Result<()> {
        let (source, args) = &self.source;
        let mut new_args = args.clone();
        new_args.push(arg.to_string());
        *self = Navigator::new(source, new_args.to_vec())?;

        Ok(())
    }

    /// Removes specified `argument`.
    pub fn remove_arg<S: std::fmt::Display>(&mut self, arg: S) -> Result<()> {
        let (source, args) = &self.source;
        let mut new_args = args.clone();
        new_args.push(arg.to_string());
        let i = args.iter().position(|x| *x == arg.to_string()).unwrap();
        new_args.remove(i);
        *self = Navigator::new(source, new_args.to_vec())?;

        Ok(())
    }

    /// Returns underlying logic program.
    pub fn program(&self) -> String {
        let (source, _) = &self.source;

        source.to_owned()
    }

    /// Resets current program to initial program.
    pub fn reset_program(&self) -> String {
        let (source, _) = &self.source;

        source.to_owned()
    }
}
impl Navigator {
    #[allow(unused)]
    fn assume(&mut self, route: &[SolverLiteral]) -> Result<()> {
        let mut ctl = self.ctl.take().ok_or(NavigatorError::NoControl)?;
        let res = ctl
            .backend()
            .and_then(|mut b| b.assume(route))
            .map_err(|e| errors::NavigatorError::Clingo(e));
        res
    }

    fn expression_to_literal(&self, expression: impl ToString) -> Option<SolverLiteral> {
        let s = expression.to_string();
        match s.starts_with('~') {
            true => lex::parse(&s[1..])
                .as_ref()
                .and_then(|symbol| self.literals.get(symbol).map(|atom| atom.negate())),
            _ => lex::parse(&s)
                .as_ref()
                .and_then(|symbol| self.literals.get(symbol))
                .copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup() {
        let nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()]);
        assert!(nav.is_ok());
    }

    #[test]
    fn enumerate() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;

        let n = nav.enumerate_solutions(None, std::iter::empty::<String>())?;
        assert_eq!(n, 3);

        let n = nav.enumerate_solutions(None, ["~c"].iter())?;
        assert_eq!(n, 2);

        let n = nav.enumerate_solutions(None, ["~d"].iter())?;
        assert_eq!(n, 2);

        let n = nav.enumerate_solutions(None, ["b"].iter())?;
        assert_eq!(n, 2);

        let n = nav.enumerate_solutions(None, ["b", "d"].iter())?;
        assert_eq!(n, 1);

        let n = nav.enumerate_solutions(None, ["a"].iter())?;
        assert_eq!(n, 1);

        let n = nav.enumerate_solutions(None, ["a", "b"].iter())?;
        assert_eq!(n, 0);

        Ok(())
    }

    #[test]
    fn enumerate_with_upper_bound() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;

        for ub in [1, 2, 3] {
            let n = nav.enumerate_solutions(Some(ub), std::iter::empty::<String>())?;
            assert_eq!(n, ub);
        }
        for ub in [4, 5, 6] {
            let n = nav.enumerate_solutions(Some(ub), std::iter::empty::<String>())?;
            assert_eq!(n, 3);
        }

        for ub in [1] {
            let n = nav.enumerate_solutions(Some(ub), ["~c"].iter())?;
            assert_eq!(n, ub);
        }
        for ub in [2, 3, 4] {
            let n = nav.enumerate_solutions(Some(ub), ["~c"].iter())?;
            assert_eq!(n, 2);
        }

        for ub in [1] {
            let n = nav.enumerate_solutions(Some(ub), ["b"].iter())?;
            assert_eq!(n, ub);
        }

        Ok(())
    }

    #[test]
    fn enumerate_outf2() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;

        let o = nav.enumerate_solutions_outf2(None, ["b"].iter())?;
        for ele in o.iter() {
            print!("{ele}")
        }

        Ok(())
    }

    #[test]
    fn enumerate_projected() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;

        nav.enumerate_projected_solutions(
            None,
            ["b"].iter(),
            vec!["c".to_string(), "d".to_string()],
        )?;

        Ok(())
    }
}
