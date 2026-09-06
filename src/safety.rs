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
        // Every operation with a dispatcher arm is serialized, including
        // `expr.eval`.
        //
        // `expr.eval` is stateless -- it reads only its own arguments -- and an
        // earlier version of this file classified it `Pure` for that reason.
        // That was wrong, because statelessness is not the property the
        // scheduler needs. `Pure` here means "a worker can run this", and a
        // worker runs a job by calling `engine::execute`, which has no
        // `expr.eval` arm: the implementation lives in `server::eval_expression`
        // and is reachable only from the request task. Classified `Pure`, a
        // mixed batch that cleared the parallel threshold returned `NYI` for the
        // `expr.eval` slot at two or more workers and the right answer at one,
        // which breaks the byte-identical-across-worker-counts invariant.
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

    /// Serialized is exactly the set of operations with a dispatcher arm, which
    /// is the set `engine::execute` cannot run.
    #[test]
    fn exactly_the_dispatcher_operations_are_serialized() {
        let serialized: Vec<&str> = registry::OPERATIONS
            .iter()
            .map(|s| s.opcode)
            .filter(|op| classify(op) == Safety::Serialized)
            .collect();
        assert_eq!(
            serialized.len(),
            8,
            "expected the 7 udo.* control operations plus expr.eval, got {serialized:?}"
        );
        assert!(serialized.iter().all(|op| op.starts_with("udo.") || *op == "expr.eval"));
        assert!(serialized.contains(&"expr.eval"),
                "expr.eval has no engine::execute arm and must never reach a worker");
    }

    /// The property the classification actually has to guarantee: anything
    /// marked Pure must be something `engine::execute` can run. Checked against
    /// the dispatcher's own control set rather than against a list kept here,
    /// so the two cannot drift apart.
    #[test]
    fn nothing_pure_has_a_dispatcher_arm() {
        for spec in registry::OPERATIONS {
            if control_op(spec.opcode).is_some() {
                assert_eq!(classify(spec.opcode), Safety::Serialized,
                           "{} has a dispatcher arm and must not run on a worker", spec.opcode);
            }
        }
    }

    #[test]
    fn the_rest_of_the_registry_is_pure() {
        let pure = registry::OPERATIONS
            .iter()
            .filter(|s| classify(s.opcode) == Safety::Pure)
            .count();
        // The structural invariant: everything without a dispatcher arm is
        // pure. This holds at any registry size.
        assert_eq!(pure, registry::OPERATIONS.len() - 8);
        // The release-line count, which moves when operations are added and is
        // here so that a change in the split is noticed rather than absorbed.
        assert_eq!(pure, 1379);
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

    /// No control operation may be pure.
    ///
    /// This test previously asserted the opposite for `expr.eval` -- that it was
    /// the one pure control operation, on the grounds that it is stateless. It
    /// is stateless, but that is not the property being classified: a worker
    /// executes a job through `engine::execute`, which has no arm for any
    /// control opcode. The old assertion pinned the bug in place rather than
    /// catching it.
    #[test]
    fn no_control_operation_is_pure() {
        let pure: Vec<&str> = ControlOp::ALL
            .iter()
            .filter(|c| classify(c.opcode()) == Safety::Pure)
            .map(|c| c.opcode())
            .collect();
        assert!(pure.is_empty(), "control operations must never run on a worker: {pure:?}");
    }
}
