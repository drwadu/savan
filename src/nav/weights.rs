use super::facets::consequences;
use super::utils::ToHashSet;
use super::Navigator;

pub fn count<S: ToString>(
    weight: &mut impl WeightingFunction,
    nav: &mut Navigator,
    peek_on: impl Iterator<Item = S>,
) -> Option<usize> {
    weight.count(nav, peek_on)
}

///
#[derive(Debug, Clone)]
pub enum Weight {
    AnswerSetCounting,
    FacetCounting,
    //SupportedModelCounting, // TODO: set config value
}

pub trait WeightingFunction {
    fn count<S: ToString>(
        &mut self,
        nav: &mut Navigator,
        peek_on: impl Iterator<Item = S>,
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

                let brave_consequences = consequences(nav, &route, "brave")?;

                match !brave_consequences.is_empty() {
                    true => consequences(nav, &route, "cautious").as_ref().and_then(
                        |cautious_consequences| {
                            Some(
                                2 * brave_consequences
                                    .difference_as_set(cautious_consequences)
                                    .iter()
                                    .count(),
                            )
                        },
                    ),
                    _ => Some(0),
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
