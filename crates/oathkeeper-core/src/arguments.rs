use crate::*;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ArgumentComparison {
    pub status: String,
    pub arguments: Vec<Report>,
}

/// Identify a supported counterargument under exactly the same frozen context.
/// This does not search for other arguments or decide legal priority.
pub fn compare_arguments(left: &Bundle, right: &Bundle) -> Result<ArgumentComparison> {
    let mut opposite = left.query.goal.clone();
    opposite.negative = !opposite.negative;
    if opposite != right.query.goal {
        return Err(Error(
            "arguments must have opposite goals for the same subject".into(),
        ));
    }
    fn context(bundle: &Bundle) -> Result<Value> {
        let mut value = serde_json::to_value(bundle).map_err(|e| Error(e.to_string()))?;
        value.as_object_mut().unwrap().remove("proof");
        let query = value["query"].as_object_mut().unwrap();
        query.remove("goal");
        query.remove("root");
        Ok(value)
    }
    if context(left)? != context(right)? {
        return Err(Error(
            "arguments must use the same frozen laws, facts, oracles, and query context".into(),
        ));
    }
    Ok(ArgumentComparison {
        status: "conflicting_arguments".into(),
        arguments: vec![verify(left)?, verify(right)?],
    })
}
