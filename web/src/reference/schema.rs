//! Nominal types and predicate signatures are part of the trusted IR, not frontend hints.
use crate::kernel::{identifier, require, unique};
use crate::*;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct Schema<'a> {
    types: BTreeSet<&'a str>,
    predicates: BTreeMap<&'a str, &'a Predicate>,
    constants: BTreeMap<&'a str, &'a Constant>,
}

impl<'a> Schema<'a> {
    pub(crate) fn new(
        types: &'a [String],
        predicates: &'a [Predicate],
        constants: &'a [Constant],
    ) -> Result<Self> {
        let mut declared = BTreeSet::new();
        for name in types {
            identifier(name)?;
            require(
                !matches!(name.as_str(), "integer" | "symbol" | "quantity")
                    && !name.starts_with("oathkeeper."),
                "reserved type name",
            )?;
            require(
                declared.insert(name.as_str()),
                format!("duplicate type: {name}"),
            )?;
        }
        let schema = Self {
            types: declared,
            predicates: unique(predicates, |p| &p.id)?,
            constants: unique(constants, |c| &c.id)?,
        };
        for predicate in predicates {
            require(
                predicate.arguments.len() <= 32,
                "[E_LIMIT] predicate arity exceeds 32",
            )?;
            require(
                !predicate.id.starts_with("oathkeeper."),
                "reserved predicate namespace",
            )?;
            for sort in &predicate.arguments {
                schema.sort(sort)?;
            }
        }
        for constant in constants {
            schema.sort(&constant.sort)?;
            require(
                !matches!(constant.sort, Sort::Integer | Sort::Quantity { .. }),
                "numeric constants must be literals",
            )?;
        }
        Ok(schema)
    }

    fn sort(&self, sort: &Sort) -> Result<()> {
        if let Sort::Quantity { unit } = sort {
            identifier(unit)?;
        }
        if let Sort::Domain(name) = sort {
            require(
                self.types.contains(name.as_str()),
                format!("[E_TYPE] unknown domain type: {name}"),
            )?;
        }
        Ok(())
    }

    fn term(&self, term: &Term, parameters: &BTreeMap<&str, &Parameter>) -> Result<Sort> {
        match term {
            Term::Integer { .. } => Ok(Sort::Integer),
            Term::Quantity { unit, .. } => {
                identifier(unit)?;
                Ok(Sort::Quantity { unit: unit.clone() })
            }
            Term::Symbol { value } => self
                .constants
                .get(value.as_str())
                .map(|c| c.sort.clone())
                .ok_or_else(|| Error(format!("[E_TYPE] undeclared constant: {value}"))),
            Term::Variable { name } => parameters
                .get(name.as_str())
                .map(|p| p.sort.clone())
                .ok_or_else(|| Error(format!("[E_TYPE] undeclared rule variable: {name}"))),
        }
    }

    pub(crate) fn claim(
        &self,
        claim: &Claim,
        parameters: &BTreeMap<&str, &Parameter>,
    ) -> Result<()> {
        let sorts = match claim.atom.as_str() {
            "oathkeeper.quantity.gt" | "oathkeeper.quantity.lt" | "oathkeeper.quantity.eq" => {
                let first = claim.arguments.first().ok_or_else(|| {
                    Error("[E_ARITY] quantity comparison requires two arguments".into())
                })?;
                let sort = self.term(first, parameters)?;
                require(
                    matches!(sort, Sort::Quantity { .. }),
                    "[E_TYPE] quantity comparison requires quantities",
                )?;
                vec![sort.clone(), sort]
            }
            "oathkeeper.quantity.add"
            | "oathkeeper.quantity.subtract"
            | "oathkeeper.quantity.allocate" => {
                let first = claim
                    .arguments
                    .first()
                    .ok_or_else(|| Error("[E_ARITY] arithmetic requires arguments".into()))?;
                let sort = self.term(first, parameters)?;
                require(
                    matches!(sort, Sort::Quantity { .. }),
                    "[E_TYPE] arithmetic requires quantities",
                )?;
                vec![
                    sort;
                    if claim.atom.ends_with("allocate") {
                        4
                    } else {
                        3
                    }
                ]
            }
            "oathkeeper.integer.gt" | "oathkeeper.integer.lt" | "oathkeeper.integer.eq" => {
                vec![Sort::Integer, Sort::Integer]
            }
            _ => self
                .predicates
                .get(claim.atom.as_str())
                .ok_or_else(|| Error(format!("[E_TYPE] undeclared predicate: {}", claim.atom)))?
                .arguments
                .clone(),
        };
        require(
            sorts.len() == claim.arguments.len(),
            format!(
                "[E_ARITY] {} expects {} arguments, received {}",
                claim.atom,
                sorts.len(),
                claim.arguments.len()
            ),
        )?;
        for (i, (expected, term)) in sorts.iter().zip(&claim.arguments).enumerate() {
            let actual = self.term(term, parameters)?;
            require(
                *expected == actual,
                format!(
                    "[E_TYPE] {} argument {} expects {expected:?}, received {actual:?}",
                    claim.atom,
                    i + 1
                ),
            )?;
        }
        Ok(())
    }

    pub(crate) fn rule(&self, rule: &Rule) -> Result<()> {
        let parameters = unique(&rule.parameters, |p| &p.name)?;
        for parameter in &rule.parameters {
            self.sort(&parameter.sort)?;
        }
        for claim in rule
            .premises
            .iter()
            .chain(std::iter::once(&rule.conclusion))
        {
            self.claim(claim, &parameters)?;
        }
        if let Some(authority) = &rule.authority {
            self.claim(&authority.scope, &parameters)?;
        }
        Ok(())
    }

    pub(crate) fn substitution(&self, rule: &Rule, values: &BTreeMap<String, Term>) -> Result<()> {
        for parameter in &rule.parameters {
            let value = values
                .get(&parameter.name)
                .ok_or_else(|| Error(format!("missing substitution: {}", parameter.name)))?;
            require(
                self.term(value, &BTreeMap::new())? == parameter.sort,
                format!("[E_TYPE] substitution sort mismatch for {}", parameter.name),
            )?;
        }
        Ok(())
    }
}

/// Static signature validation; legal eligibility and proof validity are checked by `verify`.
pub fn validate_module_schema(module: &LawModule) -> Result<()> {
    require(
        matches!(
            module.language.as_str(),
            "oathkeeper/0.3" | "oathkeeper/0.4"
        ),
        "signature validation requires oathkeeper/0.3",
    )?;
    let schema = Schema::new(&module.types, &module.predicates, &module.constants)?;
    for rule in &module.rules {
        schema
            .rule(rule)
            .map_err(|e| Error(format!("rule {}@{}: {e}", rule.id, rule.version)))?;
    }
    Ok(())
}
