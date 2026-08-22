use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaContext};

use crate::Validate;

#[derive(Debug)]
pub enum BranchApplicability {
    Applicable,
    Rejected {
        diagnostic_ranges: Vec<tombi_text::Range>,
    },
}

impl BranchApplicability {
    pub fn is_applicable(&self) -> bool {
        matches!(self, Self::Applicable)
    }

    pub fn is_recoverable_at(&self, position: tombi_text::Position) -> bool {
        match self {
            Self::Applicable => true,
            Self::Rejected { diagnostic_ranges } => {
                !diagnostic_ranges.is_empty()
                    && diagnostic_ranges
                        .iter()
                        .all(|range| range.contains(position))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Applicator {
    OneOf,
    AnyOf,
}

#[derive(Debug)]
pub struct BranchEvaluationTrace {
    pub applicator: Applicator,
    pub branches: Vec<BranchApplicability>,
}

impl BranchEvaluationTrace {
    pub fn applicable_count(&self) -> usize {
        self.branches
            .iter()
            .filter(|branch| branch.is_applicable())
            .count()
    }

    /// Branches retained while resolving a path. Ambiguous or incomplete input
    /// keeps every candidate so callers do not invent an order-dependent winner.
    pub fn includes_in_resolution(&self, index: usize, applicable_count: usize) -> bool {
        match self.applicator {
            Applicator::OneOf if applicable_count == 1 => self.branches[index].is_applicable(),
            Applicator::AnyOf if applicable_count > 0 => self.branches[index].is_applicable(),
            Applicator::OneOf | Applicator::AnyOf => true,
        }
    }
}

pub fn evaluate_applicator<'a: 'b, 'b, T>(
    applicator: Applicator,
    value: &'a T,
    accessors: &'a [Accessor],
    schemas: &'a [CurrentSchema<'a>],
    schema_context: &'a SchemaContext<'a>,
) -> tombi_future::BoxFuture<'b, BranchEvaluationTrace>
where
    T: Validate + Sync + Send + std::fmt::Debug,
{
    async move {
        let mut evaluations = Vec::with_capacity(schemas.len());
        for schema in schemas {
            let applicability = match value
                .validate(accessors, Some(schema), schema_context)
                .await
            {
                Ok(_) => BranchApplicability::Applicable,
                Err(invalid) if !invalid.assertion_failed => BranchApplicability::Applicable,
                Err(invalid) => BranchApplicability::Rejected {
                    diagnostic_ranges: invalid
                        .diagnostics
                        .into_iter()
                        .map(|diagnostic| diagnostic.range())
                        .collect(),
                },
            };
            evaluations.push(applicability);
        }
        BranchEvaluationTrace {
            applicator,
            branches: evaluations,
        }
    }
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn applicable_trace(applicator: Applicator, count: usize) -> BranchEvaluationTrace {
        BranchEvaluationTrace {
            applicator,
            branches: (0..count)
                .map(|_| BranchApplicability::Applicable)
                .collect(),
        }
    }

    #[test]
    fn one_of_keeps_all_candidates_when_multiple_branches_apply() {
        let trace = applicable_trace(Applicator::OneOf, 2);
        let count = trace.applicable_count();
        assert!(trace.includes_in_resolution(0, count));
        assert!(trace.includes_in_resolution(1, count));
    }

    #[test]
    fn one_of_selects_its_only_applicable_branch() {
        let trace = BranchEvaluationTrace {
            applicator: Applicator::OneOf,
            branches: vec![
                BranchApplicability::Rejected {
                    diagnostic_ranges: Vec::new(),
                },
                BranchApplicability::Applicable,
                BranchApplicability::Rejected {
                    diagnostic_ranges: Vec::new(),
                },
            ],
        };
        let count = trace.applicable_count();
        assert!(!trace.includes_in_resolution(0, count));
        assert!(trace.includes_in_resolution(1, count));
        assert!(!trace.includes_in_resolution(2, count));
    }

    #[test]
    fn any_of_preserves_every_applicable_branch() {
        let trace = BranchEvaluationTrace {
            applicator: Applicator::AnyOf,
            branches: vec![
                BranchApplicability::Applicable,
                BranchApplicability::Rejected {
                    diagnostic_ranges: Vec::new(),
                },
                BranchApplicability::Applicable,
            ],
        };
        let count = trace.applicable_count();
        assert!(trace.includes_in_resolution(0, count));
        assert!(!trace.includes_in_resolution(1, count));
        assert!(trace.includes_in_resolution(2, count));
    }

    #[test]
    fn rejected_branch_without_diagnostics_is_not_recoverable() {
        let applicability = BranchApplicability::Rejected {
            diagnostic_ranges: Vec::new(),
        };
        assert!(!applicability.is_recoverable_at(Default::default()));
    }
}
