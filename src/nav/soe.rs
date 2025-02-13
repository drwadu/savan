use super::super::lex;
use super::Navigator;
use crate::nav::errors::NavigatorError;
use crate::nav::utils::ToHashSet;
use std::collections::{HashMap, HashSet};

pub trait Collect {
    fn sieve(&mut self, target_atoms: &[String]) -> super::Result<()>;
    fn sieve_quiet(
        &mut self,
        target_atoms: &[String],
    ) -> Option<Vec<String>>;
    fn sieve_verbose(&mut self, target_atoms: &[String]) -> super::Result<()>;
    fn sieve_outf2(&mut self, target_atoms: &[String]) -> super::Result<Vec<String>>;
}
impl Collect for Navigator {
    fn sieve(&mut self, target_atoms: &[String]) -> super::Result<()> {
        let mut or = ":-".to_owned();
        target_atoms.iter().for_each(|atom| {
            or = format!("{or} not {atom},");
        });

        or = format!("{}.", &or[..or.len() - 1]);
        self.add_rule(or.clone())?;

        let mut i = 1;
        let mut to_observe = target_atoms.to_vec().to_hashset();
        let mut collection = vec![].to_hashset();

        while !to_observe.is_empty() {
            let target = to_observe
                .iter()
                .next()
                .and_then(|a| self.expression_to_literal(a))
                .ok_or(NavigatorError::None)?;

            let ctl = self.ctl.take().ok_or(NavigatorError::NoControl)?;
            let mut solve_handle = ctl.solve(clingo::SolveMode::YIELD, &[target])?;

            if solve_handle
                .get()
                .map(|r| r == clingo::SolveResult::SATISFIABLE)?
                == false
            {
                println!("info: cannot cover all target atoms");
                println!("info: stopped search");
                break;
            }

            #[allow(clippy::needless_collect)]
            while let Ok(Some(model)) = solve_handle.model() {
                if let Ok(atoms) = model.symbols(clingo::ShowType::SHOWN) {
                    match atoms
                        .iter()
                        .map(|a| to_observe.remove(&a.to_string()))
                        .collect::<Vec<_>>()
                        .iter()
                        .any(|v| *v)
                    {
                        true => {
                            if collection.insert(atoms.clone()) {
                                println!("solution {:?}: ", i);
                                let atoms_strings = atoms.iter().map(|atom| atom.to_string());
                                atoms_strings.clone().for_each(|atom| print!("{} ", atom));
                                i += 1;
                                println!();

                                break;
                            }
                        }
                        _ => {
                            solve_handle.resume()?;
                            continue;
                        } // did not observe anything new
                    }
                }
            }

            let ctl = solve_handle
                .close()
                .map_err(|e| NavigatorError::Clingo(e))?;
            self.ctl = Some(ctl);
        }

        self.remove_rule(or)
    }

    fn sieve_quiet(
        &mut self,
        target_atoms: &[String],
    ) -> Option<Vec<String>> {
        let mut or = ":-".to_owned();
        target_atoms.iter().for_each(|atom| {
            or = format!("{or} not {atom},");
        });

        or = format!("{}.", &or[..or.len() - 1]);
        self.add_rule(or.clone()).ok()?;

        let mut to_observe = target_atoms.to_vec().to_hashset();
        let mut true_somewhere = vec![];

        while !to_observe.is_empty() {
            let (target_atom, target) = to_observe
                .iter()
                .next()
                .map(|a| (a, self.expression_to_literal(a).unwrap()))?;

            let ctl = self.ctl.take()?;
            let mut solve_handle = ctl.solve(clingo::SolveMode::YIELD, &[target]).ok()?;

            if solve_handle
                .get()
                .map(|r| r == clingo::SolveResult::SATISFIABLE)
                .ok()?
                == false
            {
                break;
            }

            #[allow(clippy::needless_collect)]
            while let Ok(Some(model)) = solve_handle.model() {
                if let Ok(atoms) = model.symbols(clingo::ShowType::SHOWN) {
                    match atoms
                        .iter()
                        .map(|a| {
                            let v = to_observe.remove(&a.to_string());
                            if v {
                                true_somewhere.push(a.to_string())
                            }
                            v
                        })
                        .collect::<Vec<_>>()
                        .iter()
                        .any(|v| *v)
                    {
                        true => {
                            break;
                        }
                        _ => {
                            solve_handle.resume().ok()?;
                            continue;
                        } // did not observe anything new
                    }
                }
            }

            let ctl = solve_handle.close().ok()?;
            self.ctl = Some(ctl);
        }

        self.remove_rule(or).ok()?;

        Some(true_somewhere)
    }

    fn sieve_outf2(&mut self, target_atoms: &[String]) -> super::Result<Vec<String>> {
        let mut out = vec![];

        let mut or = ":-".to_owned();
        target_atoms.iter().for_each(|atom| {
            or = format!("{or} not {atom},");
        });

        or = format!("{}.", &or[..or.len() - 1]);
        self.add_rule(or.clone())?;

        let mut i = 1;
        let mut to_observe = target_atoms.to_vec().to_hashset();
        let mut collection = vec![].to_hashset();

        while !to_observe.is_empty() {
            let target = to_observe
                .iter()
                .next()
                .and_then(|a| self.expression_to_literal(a))
                .ok_or(NavigatorError::None)?;

            let ctl = self.ctl.take().ok_or(NavigatorError::NoControl)?;
            let mut solve_handle = ctl.solve(clingo::SolveMode::YIELD, &[target])?;

            if solve_handle
                .get()
                .map(|r| r == clingo::SolveResult::SATISFIABLE)?
                == false
            {
                println!("info: cannot cover all target atoms");
                println!("info: stopped search");
                break;
            }

            #[allow(clippy::needless_collect)]
            while let Ok(Some(model)) = solve_handle.model() {
                if let Ok(atoms) = model.symbols(clingo::ShowType::SHOWN) {
                    match atoms
                        .iter()
                        .map(|a| to_observe.remove(&a.to_string()))
                        .collect::<Vec<_>>()
                        .iter()
                        .any(|v| *v)
                    {
                        true => {
                            if collection.insert(atoms.clone()) {
                                println!("solution {:?}: ", i);
                                let mut answer_set =
                                    "{\"Solver\": \"\", \"Input\": [\"\"], ".to_owned();
                                answer_set = format!(
                                    "{answer_set}\"Call\": [ {{ \"Witnesses\": [ {{ \"Value\": ["
                                );
                                if let Some((last, rest)) = atoms.split_last() {
                                    for atom in rest {
                                        answer_set =
                                            format!("{answer_set}{:?}, ", atom.to_string());
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
                                println!();

                                break;
                            }
                        }
                        _ => {
                            solve_handle.resume()?;
                            continue;
                        } // did not observe anything new
                    }
                }
            }

            let ctl = solve_handle
                .close()
                .map_err(|e| NavigatorError::Clingo(e))?;
            self.ctl = Some(ctl);
        }

        self.remove_rule(or)?;

        Ok(out)
    }

    fn sieve_verbose(&mut self, target_atoms: &[String]) -> super::Result<()> {
        let mut or = ":-".to_owned();
        target_atoms.iter().for_each(|atom| {
            or = format!("{or} not {atom},");
        });

        or = format!("{}.", &or[..or.len() - 1]);
        self.add_rule(or.clone())?;

        let mut n = 0;
        let mut freq_table: HashMap<clingo::Symbol, usize> = HashMap::new();
        target_atoms.iter().for_each(|atom| {
            n += 1;
            freq_table.insert(unsafe { lex::parse(&atom).unwrap_unchecked() }, 0);
        });
        let mut chunks_table: HashMap<usize, HashSet<clingo::Symbol>> = HashMap::new();
        let mut population_size = 0;
        let mut i = 1;
        let mut to_observe = target_atoms.to_vec().to_hashset();
        let mut collection = vec![].to_hashset();

        while !to_observe.is_empty() {
            let target = to_observe
                .iter()
                .next()
                .and_then(|a| self.expression_to_literal(a))
                .ok_or(NavigatorError::None)?;

            let ctl = self.ctl.take().ok_or(NavigatorError::NoControl)?;
            let mut solve_handle = ctl.solve(clingo::SolveMode::YIELD, &[target])?;

            if solve_handle
                .get()
                .map(|r| r == clingo::SolveResult::SATISFIABLE)?
                == false
            {
                println!("info: cannot cover all target atoms");
                println!("info: stopped search");
                break;
            }

            #[allow(clippy::needless_collect)]
            while let Ok(Some(model)) = solve_handle.model() {
                if let Ok(atoms) = model.symbols(clingo::ShowType::SHOWN) {
                    match atoms
                        .iter()
                        .map(|a| to_observe.remove(&a.to_string()))
                        .collect::<Vec<_>>()
                        .iter()
                        .any(|v| *v)
                    {
                        true => {
                            if collection.insert(atoms.clone()) {
                                atoms.iter().for_each(|atom| {
                                    if let Some(count) = freq_table.get_mut(atom) {
                                        *count += 1;
                                    }
                                });

                                println!("solution {:?}: ", i);
                                let atoms_strings = atoms.iter().map(|atom| atom.to_string());
                                atoms_strings.clone().for_each(|atom| print!("{} ", atom));
                                i += 1;
                                println!();

                                break;
                            }
                        }
                        _ => {
                            solve_handle.resume()?;
                            continue;
                        } // did not observe anything new
                    }
                }
            }

            let ctl = solve_handle
                .close()
                .map_err(|e| NavigatorError::Clingo(e))?;
            self.ctl = Some(ctl);
        }

        freq_table.iter().for_each(|(atom, freq)| {
            population_size += *freq;
            //let freq_chunk = chunks_table
            //    .raw_entry_mut()
            //    .from_key(freq)
            //    .or_insert_with(|| (*freq, vec![*atom].to_hashset()));
            //freq_chunk.1.insert(*atom);
            chunks_table
                .entry(*freq)
                .and_modify(|s| {
                    s.insert(*atom);
                })
                .or_insert_with(|| vec![*atom].to_hashset());
        });
        let div = 2f64.powf(entropy(&freq_table, population_size as f64));
        let r = {
            let ts = n as f64;
            1f64 - (ts - div).abs() / ts
        };

        println!("-");
        let c = (i - 1) as f64;
        freq_table
            .iter()
            .for_each(|(k, v)| println!("{:.2} {}", *v as f64 / c, k.to_string()));
        println!(
            "{:?} {:?}",
            freq_table.values().filter(|v| **v != 0).count() as f64 / n as f64,
            r
        );
        println!("-");
        self.remove_rule(or)
    }
}

fn entropy(lookup_table: &HashMap<clingo::Symbol, usize>, sample_size: f64) -> f64 {
    -lookup_table
        .iter()
        .map(|(_, count)| *count as f64 / sample_size)
        .map(|probability| probability * probability.log2())
        .sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::super::errors::Result;
    use super::*;

    #[test]
    fn soe_sieve() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        nav.sieve(&["a".to_owned(), "b".to_owned()])?;
        nav.sieve_verbose(&["a".to_owned(), "b".to_owned()])
    }
}
