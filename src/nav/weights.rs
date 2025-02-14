use super::facets::{consequences_count, consequences_count_projecting};
use super::Navigator;

/// Returns count of specified weighting function under route.
pub fn count<S: ToString>(
    weighting_function: &mut impl WeightingFunction,
    nav: &mut Navigator,
    route: impl Iterator<Item = S>,
) -> Option<usize> {
    weighting_function.count(nav, route)
}

/// Returns count of specified weighting function under route, while projecting on shown atoms.
pub fn count_projecting<S: ToString>(
    weighting_function: &mut impl WeightingFunction,
    nav: &mut Navigator,
    route: impl Iterator<Item = S>,
) -> Option<usize> {
    weighting_function.count_projecting(nav, route)
}

/// The weight of a facet.
#[derive(Debug, Clone)]
pub enum Weight {
    AnswerSetCounting,
    FacetCounting,
}

/// Implements counting procedures.
pub trait WeightingFunction {
    fn count<S: ToString>(
        &mut self,
        nav: &mut Navigator,
        route: impl Iterator<Item = S>,
    ) -> Option<usize>;
    fn count_projecting<S: ToString>(
        &mut self,
        nav: &mut Navigator,
        route: impl Iterator<Item = S>,
    ) -> Option<usize>;
}
impl WeightingFunction for Weight {
    fn count<S: ToString>(
        &mut self,
        nav: &mut Navigator,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<usize> {
        match self {
            Self::FacetCounting => {
                let route = peek_on
                    .map(|s| nav.expression_to_literal(s))
                    .flatten()
                    .collect::<Vec<_>>();

                let brave_consequences_count = consequences_count(nav, &route, "brave");
                if brave_consequences_count == Some(0) {
                    brave_consequences_count
                } else {
                    brave_consequences_count.and_then(|bcs| {
                        consequences_count(nav, &route, "cautious").map(|ccs| 2 * (bcs - ccs))
                    })
                }
            }
            Self::AnswerSetCounting => nav.enumerate_solutions_quietly(None, peek_on).ok(),
        }
    }
    fn count_projecting<S: ToString>(
        &mut self,
        nav: &mut Navigator,
        peek_on: impl Iterator<Item = S>,
    ) -> Option<usize> {
        match self {
            Self::FacetCounting => {
                let route = peek_on
                    .map(|s| nav.expression_to_literal(s))
                    .flatten()
                    .collect::<Vec<_>>();

                let brave_consequences_count = consequences_count_projecting(nav, &route, "brave");
                if brave_consequences_count == Some(0) {
                    brave_consequences_count
                } else {
                    brave_consequences_count.and_then(|bcs| {
                        consequences_count_projecting(nav, &route, "cautious")
                            .map(|ccs| 2 * (bcs - ccs))
                    })
                }
            }
            Self::AnswerSetCounting => nav.enumerate_solutions_quietly(None, peek_on).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::errors::Result;
    use super::*;
    use crate::nav::errors::NavigatorError;

    #[test]
    fn facet_count() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let mut w = Weight::FacetCounting;

        let c = count(&mut w, &mut nav, ["a", "b"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 0);

        let c = count(&mut w, &mut nav, ["a"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 0);

        let c = count(&mut w, &mut nav, ["c"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 0);

        let c = count(&mut w, &mut nav, ["d"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 0);

        let c = count(&mut w, &mut nav, ["~b"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 0);

        let c = count(&mut w, &mut nav, ["b"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 4);

        let c = count(&mut w, &mut nav, ["~a"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 4);

        let c = count(&mut w, &mut nav, ["~c"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 6);

        let c = count(&mut w, &mut nav, ["~d"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 6);

        let c =
            count(&mut w, &mut nav, std::iter::empty::<String>()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 8);

        Ok(())
    }

    #[test]
    fn answer_set_count() -> Result<()> {
        let mut nav = Navigator::new("a;b. c;d :- b. e.", vec!["0".to_string()])?;
        let mut w = Weight::AnswerSetCounting;

        let c = count(&mut w, &mut nav, ["a", "b"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 0);

        let c = count(&mut w, &mut nav, ["a"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 1);

        let c = count(&mut w, &mut nav, ["c"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 1);

        let c = count(&mut w, &mut nav, ["d"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 1);

        let c = count(&mut w, &mut nav, ["~b"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 1);

        let c = count(&mut w, &mut nav, ["b"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 2);

        let c = count(&mut w, &mut nav, ["~a"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 2);

        let c = count(&mut w, &mut nav, ["~c"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 2);

        let c = count(&mut w, &mut nav, ["~d"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 2);

        let c =
            count(&mut w, &mut nav, std::iter::empty::<String>()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 3);

        Ok(())
    }

    #[test]
    fn supported_model_count() -> Result<()> {
        let mut nav = Navigator::new(
            "a :- b. b :- a. a :- c. c :- not d. d :- not c.",
            vec!["0".to_string(), "--supp-models".to_string()],
        )?;

        // Answer: 1
        // d
        // Answer: 2
        // d a b  # not stable
        // Answer: 3
        // c a b
        // SATISFIABLE

        let mut w = Weight::AnswerSetCounting;

        let c =
            count(&mut w, &mut nav, std::iter::empty::<String>()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 3);

        let c = count(&mut w, &mut nav, ["~d"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 1);

        let c = count(&mut w, &mut nav, ["d"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 2);

        let c = count(&mut w, &mut nav, ["b"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 2);

        let c = count(&mut w, &mut nav, ["a"].iter()).ok_or(NavigatorError::None)?;
        assert_eq!(c, 2);

        Ok(())
    }
}
