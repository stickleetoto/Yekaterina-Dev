//! Internal execution-safety classification.
//!
//! Decides what may run concurrently on a worker thread and what must not.
//! **Never exposed through MCP**: no tool, no `yk.spec` field, no schema change.
//!
//! # FAIL_CLOSED
//!
//! The default is [`Safety::Serialized`]. An opcode that is not registered --
//! a user formula, a composite, a pack operation, a typo -- is serialized. An
//! operation is only ever classified [`Safety::Pure`] because of a positive
//! structural fact about how the dispatcher routes it, never because nothing
//! matched.
//!
//! # Why this is sound
//!
//! ```ignore
//! pub fn execute(opcode: &str, args: &[Value]) -> Result<Value, &'static str>
//! ```
//!
//! `engine::execute` takes no `&self` and no state parameter, so an operation
//! routed there cannot reach the user registry, the filesystem, or anything else
//! the server owns. That is a type guarantee.
//!
//! The server dispatcher handles exactly the [`ControlOp`] opcodes itself and
//! delegates everything else to that function. Statefulness therefore *requires*
//! a dispatcher arm, and a dispatcher arm requires a `ControlOp` variant: a
//! stateful operation that someone forgot to classify would be routed to
//! `engine::execute` and simply not work, rather than silently racing.
//!
//! `server.rs` matches on [`control_op`] directly, so the classifier and the
//! dispatcher are the same code and cannot drift apart.
//!
//! Deliberately contains no prefix matching. `registry::capability_code` and
//! `registry::cost_code` classify with `starts_with` for presentation, which
//! would silently misclassify a future operation if used for scheduling.
//!
//! See `docs/V11_SAFETY_MODEL.md` for the options considered and rejected.

use crate::registry;

/// Execution-safety class of one operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    /// May execute on a worker thread, concurrently with other pure work.
    /// Reachable only through `engine::execute`, which cannot touch state.
    Pure,
    /// Must execute on the request task, in order. Either it mutates
    /// server-owned state, reads it, or its safety is unknown.
    Serialized,
}

impl Safety {
    pub fn is_pure(self) -> bool {
        matches!(self, Safety::Pure)
    }
}

/// The operations the server dispatcher handles itself rather than delegating
/// to `engine::execute`.
///
/// This enum is the single source of truth: `server.rs` dispatches on it, and
/// [`classify`] derives safety from it. Adding a control operation without a
/// variant here is caught by `scripts/static_audit_v11.py`, which asserts every
/// registered `udo.*` opcode appears below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlOp {
    /// `udo.formula` — defines a user formula. Mutates the registry.
    DefineFormula,
    /// `udo.composite` — defines a user composite. Mutates the registry.
    DefineComposite,
    /// `udo.remove` — removes a user operation. Mutates the registry.
    Remove,
    /// `udo.import` — installs a pack. Mutates the registry.
    Import,
    /// `udo.uninstall` — removes a pack. Mutates the registry.
    Uninstall,
    /// `udo.list` — reads the registry snapshot.
    List,
    /// `udo.export` — reads the registry snapshot.
    Export,
    /// `expr.eval` — handled in the server but reads only its own arguments.
    ExprEval,
}

impl ControlOp {
    /// Canonical opcode for this control operation.
    pub fn opcode(self) -> &'static str {
        match self {
            ControlOp::DefineFormula => "udo.formula",
            ControlOp::DefineComposite => "udo.composite",
            ControlOp::Remove => "udo.remove",
            ControlOp::Import => "udo.import",
            ControlOp::Uninstall => "udo.uninstall",
            ControlOp::List => "udo.list",
            ControlOp::Export => "udo.export",
            ControlOp::ExprEval => "expr.eval",
        }
    }

    /// Every control operation, for exhaustiveness tests.
    pub const ALL: &'static [ControlOp] = &[
        ControlOp::DefineFormula,
        ControlOp::DefineComposite,
        ControlOp::Remove,
        ControlOp::Import,
        ControlOp::Uninstall,
        ControlOp::List,
        ControlOp::Export,
        ControlOp::ExprEval,
    ];

    /// Whether this operation changes server-owned state.
    pub fn mutates(self) -> bool {
        matches!(
            self,
            ControlOp::DefineFormula
                | ControlOp::DefineComposite
                | ControlOp::Remove
                | ControlOp::Import
                | ControlOp::Uninstall
        )
    }
}

/// Map a **canonical** opcode to its control operation, if it has one.
///
/// Callers that may hold an alias or a differently-cased spelling must
/// canonicalise through `registry::resolve` first, exactly as the dispatcher
/// does. [`classify`] handles that.
pub fn control_op(canonical: &str) -> Option<ControlOp> {
    Some(match canonical {
        "udo.formula" => ControlOp::DefineFormula,
        "udo.composite" => ControlOp::DefineComposite,
        "udo.remove" => ControlOp::Remove,
        "udo.import" => ControlOp::Import,
        "udo.uninstall" => ControlOp::Uninstall,
        "udo.list" => ControlOp::List,
        "udo.export" => ControlOp::Export,
        "expr.eval" => ControlOp::ExprEval,
        _ => return None,
    })
}

/// Classify an opcode. Accepts aliases and any casing that `registry::resolve`
/// accepts, and canonicalises the same way the dispatcher does.
///
/// FAIL_CLOSED: unregistered opcodes are [`Safety::Serialized`].
pub fn classify(opcode: &str) -> Safety {
    let Some(spec) = registry::resolve(opcode) else {
        // Not a built-in: a user formula or composite, a pack operation, or an
        // unknown opcode. All serialized until proven otherwise.
        return Safety::Serialized;
    };
    match control_op(spec.opcode) {
        // Stateless despite having a dispatcher arm: reads only its arguments.
        Some(ControlOp::ExprEval) => Safety::Pure,
        // Mutates or reads server-owned state.
        Some(_) => Safety::Serialized,
        // No dispatcher arm => routed to `engine::execute`, which by its
        // signature cannot reach state.
        None => Safety::Pure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A new control operation cannot be added without classifying it: every
    /// registered `udo.*` opcode must have a `ControlOp` variant.
    #[test]
    fn every_registered_udo_opcode_is_a_control_op() {
        let missing: Vec<&str> = registry::OPERATIONS
            .iter()
            .map(|s| s.opcode)
            .filter(|op| op.starts_with("udo.") && control_op(op).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "registered udo.* opcodes with no ControlOp variant: {missing:?}"
        );
    }

    /// And no dead entries pointing at opcodes that do not exist.
    #[test]
    fn every_control_op_is_registered() {
        for c in ControlOp::ALL {
            assert!(
                registry::resolve(c.opcode()).is_some(),
                "ControlOp {:?} names unregistered opcode {}",
                c,
                c.opcode()
            );
            assert_eq!(
                registry::resolve(c.opcode()).unwrap().opcode,
                c.opcode(),
                "ControlOp {c:?} opcode is not canonical"
            );
        }
    }

    #[test]
    fn exactly_seven_registered_operations_are_serialized() {
        let serialized: Vec<&str> = registry::OPERATIONS
            .iter()
            .map(|s| s.opcode)
            .filter(|op| classify(op) == Safety::Serialized)
            .collect();
        assert_eq!(
            serialized.len(),
            7,
            "expected the 7 udo.* control operations, got {serialized:?}"
        );
        assert!(serialized.iter().all(|op| op.starts_with("udo.")));
    }

    #[test]
    fn the_rest_of_the_registry_is_pure() {
        let pure = registry::OPERATIONS
            .iter()
            .filter(|s| classify(s.opcode) == Safety::Pure)
            .count();
        assert_eq!(pure, registry::OPERATIONS.len() - 7);
        assert_eq!(pure, 1208);
    }

    /// FAIL_CLOSED. Nothing outside the static registry may be pure.
    #[test]
    fn unknown_and_dynamic_operations_are_serialized() {
        for op in [
            "",
            "   ",
            "zzz.nope",
            "user.double",
            "user.anything",
            "pack.demo.thing",
            "udo.reset",          // plausible future control op, not yet added
            "math.add.extra",
            "\u{1F600}",
        ] {
            assert_eq!(
                classify(op),
                Safety::Serialized,
                "{op:?} must be serialized under the fail-closed default"
            );
        }
    }

    /// The classifier canonicalises exactly as the dispatcher does, so an alias
    /// or a differently-cased spelling cannot classify differently.
    #[test]
    fn aliases_and_casing_classify_as_their_canonical_opcode() {
        for (spelling, canonical) in [
            ("avg", "stat.mean"),
            ("AVG", "stat.mean"),
            ("Math.Add", "math.add"),
            ("  math.add  ", "math.add"),
            ("+", "math.add"),
        ] {
            assert_eq!(
                registry::resolve(spelling).unwrap().opcode,
                canonical,
                "test premise: {spelling} resolves to {canonical}"
            );
            assert_eq!(classify(spelling), classify(canonical), "{spelling}");
        }
        // Control operations reached through any accepted spelling stay serialized.
        for spelling in ["udo.formula", "UDO.FORMULA", "  udo.formula "] {
            assert_eq!(classify(spelling), Safety::Serialized, "{spelling}");
        }
    }

    #[test]
    fn mutating_control_ops_are_exactly_the_registry_writers() {
        let mutating: Vec<&str> = ControlOp::ALL
            .iter()
            .filter(|c| c.mutates())
            .map(|c| c.opcode())
            .collect();
        assert_eq!(
            mutating,
            vec![
                "udo.formula",
                "udo.composite",
                "udo.remove",
                "udo.import",
                "udo.uninstall"
            ]
        );
    }

    #[test]
    fn expr_eval_is_the_only_pure_control_op() {
        let pure: Vec<&str> = ControlOp::ALL
            .iter()
            .filter(|c| classify(c.opcode()) == Safety::Pure)
            .map(|c| c.opcode())
            .collect();
        assert_eq!(pure, vec!["expr.eval"]);
    }
}
