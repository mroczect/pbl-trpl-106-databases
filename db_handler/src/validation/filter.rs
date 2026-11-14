use crate::types::core::{Condition, Filter};
use crate::validation::validate_identifier;
use crate::{DbError, Result};

pub fn validate_filter(filter: &Filter) -> Result<()> {
    for cond in &filter.conditions {
        validate_condition(cond)?;
    }
    Ok(())
}

fn validate_condition(cond: &Condition) -> Result<()> {
    match cond {
        Condition::Eq { column, .. }
        | Condition::Ne { column, .. }
        | Condition::Lt { column, .. }
        | Condition::Le { column, .. }
        | Condition::Gt { column, .. }
        | Condition::Ge { column, .. }
        | Condition::In { column, .. }
        | Condition::IsNull { column }
        | Condition::IsNotNull { column } => validate_identifier(column),
        Condition::And(children) | Condition::Or(children) => {
            if children.is_empty() {
                return Err(DbError::Validation {
                    message: "nested AND/OR must have at least one condition".into(),
                });
            }
            for c in children {
                validate_condition(c)?;
            }
            Ok(())
        }
    }
}
