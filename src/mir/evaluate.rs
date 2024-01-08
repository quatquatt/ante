//! Evaluate any compile-time function applications in the Hir to remove handler abstractions
use crate::util::fmap;

use super::ir::{ self as mir, Ast, dispatch_on_mir, DefinitionId, Atom, Mir };

impl Mir {
    pub fn evaluate_static_calls(mut self) -> Mir {
        self.functions = self.functions.into_iter().map(|(id, function)| {
            (id, function.evaluate(&im::HashMap::new()))
        }).collect();
        self
    }
}

type Substitutions = im::HashMap<DefinitionId, Atom>;

/// Evaluate static calls in `self` using the given substitutions
trait Evaluate<T> {
    fn evaluate(self, substitutions: &Substitutions) -> T;
}

impl Evaluate<Ast> for Ast {
    fn evaluate(self, substitutions: &Substitutions) -> Ast {
        dispatch_on_mir!(self, Evaluate::evaluate, substitutions)
    }
}

impl Evaluate<Atom> for Atom {
    fn evaluate(self, substitutions: &Substitutions) -> Atom {
        dispatch_on_atom!(self, Evaluate::evaluate, substitutions)
    }
}

impl Evaluate<Ast> for Atom {
    fn evaluate(self, substitutions: &Substitutions) -> Ast {
        Ast::Atom(self.evaluate(substitutions))
    }
}

impl Evaluate<Atom> for mir::Literal {
    fn evaluate(self, _: &Substitutions) -> Atom {
        Atom::Literal(self)
    }
}

impl Evaluate<Atom> for mir::Variable {
    fn evaluate(self, substitutions: &Substitutions) -> Atom {
        match substitutions.get(&self.definition_id) {
            Some(ast) => ast.clone(), // Should we recur here?
            None => Atom::Variable(self),
        }
    }
}

impl Evaluate<Atom> for mir::Lambda {
    // Any variables introduced by the lambda shadow any matching variables in `substitutions`,
    // so make sure to remove them before evaluating the lambda body.
    fn evaluate(mut self, substitutions: &Substitutions) -> Atom {
        let mut substitutions = substitutions.clone();

        for arg in &self.args {
            substitutions.remove(&arg.definition_id);
        }

        *self.body = self.body.evaluate(&substitutions);
        Atom::Lambda(self)
    }
}

impl Evaluate<Atom> for mir::Extern {
    fn evaluate(self, _: &Substitutions) -> Atom {
        Atom::Extern(self)
    }
}

impl Evaluate<Ast> for mir::FunctionCall {
    fn evaluate(mut self, substitutions: &Substitutions) -> Ast {
        let function = self.function.evaluate(substitutions);
        let args = fmap(self.args, |arg| arg.evaluate(substitutions));

        match function {
            Atom::Lambda(lambda) if lambda.compile_time || self.compile_time => {
                let mut new_substitutions = substitutions.clone();
                assert_eq!(lambda.args.len(), args.len());

                for (param, arg) in lambda.args.iter().zip(args) {
                    new_substitutions.insert(param.definition_id, arg);
                }

                lambda.body.evaluate(&new_substitutions).evaluate(substitutions)
            }
            function => {
                self.function = function;
                self.args = args;
                Ast::FunctionCall(self)
            }
        }
    }
}

impl Evaluate<Ast> for mir::Let<Ast> {
    fn evaluate(mut self, substitutions: &Substitutions) -> Ast {
        *self.expr = self.expr.evaluate(substitutions);
        *self.body = self.body.evaluate(substitutions);
        Ast::Let(self)
    }
}

impl Evaluate<Ast> for mir::If {
    fn evaluate(mut self, substitutions: &Substitutions) -> Ast {
        self.condition = self.condition.evaluate(substitutions);
        *self.then = self.then.evaluate(substitutions);
        *self.otherwise = self.otherwise.evaluate(substitutions);
        Ast::If(self)
    }
}

impl Evaluate<Ast> for mir::Match {
    fn evaluate(mut self, substitutions: &Substitutions) -> Ast {
        self.decision_tree = evaluate_decision_tree(self.decision_tree, substitutions);
        self.branches = fmap(self.branches, |branch| branch.evaluate(substitutions));
        Ast::Match(self)
    }
}

fn evaluate_decision_tree(tree: mir::DecisionTree, substitutions: &Substitutions) -> mir::DecisionTree {
    match tree {
        mir::DecisionTree::Leaf(_) => todo!(),
        mir::DecisionTree::Let(_) => todo!(),
        mir::DecisionTree::Switch { int_to_switch_on, cases, else_case } => todo!(),
    }
}

impl Evaluate<Ast> for mir::Return {
    fn evaluate(mut self, substitutions: &Substitutions) -> Ast {
        self.expression = self.expression.evaluate(substitutions);
        Ast::Return(self)
    }
}

impl Evaluate<Ast> for mir::Assignment {
    fn evaluate(mut self, substitutions: &Substitutions) -> Ast {
        self.lhs = self.lhs.evaluate(substitutions);
        self.rhs = self.rhs.evaluate(substitutions);
        Ast::Assignment(self)
    }
}

impl Evaluate<Ast> for mir::MemberAccess {
    fn evaluate(mut self, substitutions: &Substitutions) -> Ast {
        self.lhs = self.lhs.evaluate(substitutions);
        Ast::MemberAccess(self)
    }
}

impl Evaluate<Ast> for mir::Tuple {
    fn evaluate(mut self, substitutions: &Substitutions) -> Ast {
        self.fields = fmap(self.fields, |field| field.evaluate(substitutions));
        Ast::Tuple(self)
    }
}

impl Evaluate<Ast> for mir::Builtin {
    fn evaluate(self, substitutions: &Substitutions) -> Ast {
        use mir::Builtin;

        let both = |f: fn(_, _) -> Builtin, lhs: Atom, rhs: Atom| {
            let lhs = lhs.evaluate(substitutions);
            let rhs = rhs.evaluate(substitutions);
            Ast::Builtin(f(lhs, rhs))
        };

        let one_with_type = |f: fn(_, _) -> Builtin, lhs: Atom, typ| {
            let lhs = lhs.evaluate(substitutions);
            Ast::Builtin(f(lhs, typ))
        };

        let one = |f: fn(_) -> Builtin, lhs: Atom| {
            let lhs = lhs.evaluate(substitutions);
            Ast::Builtin(f(lhs))
        };

        match self {
            Builtin::AddInt(lhs, rhs) => both(Builtin::AddInt, lhs, rhs),
            Builtin::AddFloat(lhs, rhs) => both(Builtin::AddFloat, lhs, rhs),
            Builtin::SubInt(lhs, rhs) => both(Builtin::SubInt, lhs, rhs),
            Builtin::SubFloat(lhs, rhs) => both(Builtin::SubFloat, lhs, rhs),
            Builtin::MulInt(lhs, rhs) => both(Builtin::MulInt, lhs, rhs),
            Builtin::MulFloat(lhs, rhs) => both(Builtin::MulFloat, lhs, rhs),
            Builtin::DivSigned(lhs, rhs) => both(Builtin::DivSigned, lhs, rhs),
            Builtin::DivUnsigned(lhs, rhs) => both(Builtin::DivUnsigned, lhs, rhs),
            Builtin::DivFloat(lhs, rhs) => both(Builtin::DivFloat, lhs, rhs),
            Builtin::ModSigned(lhs, rhs) => both(Builtin::ModSigned, lhs, rhs),
            Builtin::ModUnsigned(lhs, rhs) => both(Builtin::ModUnsigned, lhs, rhs),
            Builtin::ModFloat(lhs, rhs) => both(Builtin::ModFloat, lhs, rhs),
            Builtin::LessSigned(lhs, rhs) => both(Builtin::LessSigned, lhs, rhs),
            Builtin::LessUnsigned(lhs, rhs) => both(Builtin::LessUnsigned, lhs, rhs),
            Builtin::LessFloat(lhs, rhs) => both(Builtin::LessFloat, lhs, rhs),
            Builtin::EqInt(lhs, rhs) => both(Builtin::EqInt, lhs, rhs),
            Builtin::EqFloat(lhs, rhs) => both(Builtin::EqFloat, lhs, rhs),
            Builtin::EqChar(lhs, rhs) => both(Builtin::EqChar, lhs, rhs),
            Builtin::EqBool(lhs, rhs) => both(Builtin::EqBool, lhs, rhs),
            Builtin::SignExtend(lhs, rhs) => one_with_type(Builtin::SignExtend, lhs, rhs),
            Builtin::ZeroExtend(lhs, rhs) => one_with_type(Builtin::ZeroExtend, lhs, rhs),
            Builtin::SignedToFloat(lhs, rhs) => one_with_type(Builtin::SignedToFloat, lhs, rhs),
            Builtin::UnsignedToFloat(lhs, rhs) => one_with_type(Builtin::UnsignedToFloat, lhs, rhs),
            Builtin::FloatToSigned(lhs, rhs) => one_with_type(Builtin::FloatToSigned, lhs, rhs),
            Builtin::FloatToUnsigned(lhs, rhs) => one_with_type(Builtin::FloatToUnsigned, lhs, rhs),
            Builtin::FloatPromote(lhs, rhs) => one_with_type(Builtin::FloatPromote, lhs, rhs),
            Builtin::FloatDemote(lhs, rhs) => one_with_type(Builtin::FloatDemote, lhs, rhs),
            Builtin::BitwiseAnd(lhs, rhs) => both(Builtin::BitwiseAnd, lhs, rhs),
            Builtin::BitwiseOr(lhs, rhs) => both(Builtin::BitwiseOr, lhs, rhs),
            Builtin::BitwiseXor(lhs, rhs) => both(Builtin::BitwiseXor, lhs, rhs),
            Builtin::BitwiseNot(lhs) => one(Builtin::BitwiseNot, lhs),
            Builtin::StackAlloc(lhs) => one(Builtin::StackAlloc, lhs),
            Builtin::Truncate(lhs, rhs) => one_with_type(Builtin::Truncate, lhs, rhs),
            Builtin::Deref(lhs, rhs) => one_with_type(Builtin::Deref, lhs, rhs),
            Builtin::Transmute(lhs, rhs) => one_with_type(Builtin::Transmute, lhs, rhs),
            Builtin::Offset(lhs, rhs, typ) => {
                let lhs = lhs.evaluate(substitutions);
                let rhs = rhs.evaluate(substitutions);
                Ast::Builtin(Builtin::Offset(lhs, rhs, typ))
            },
        }
    }
}

impl Evaluate<Atom> for mir::Effect {
    fn evaluate(self, _: &Substitutions) -> Atom {
        unreachable!("Effect nodes should be removed by the mir-cps pass before evaluation")
    }
}

impl Evaluate<Ast> for mir::Handle {
    fn evaluate(self, _: &Substitutions) -> Ast {
        unreachable!("Handle expressions should be removed by the mir-cps pass before evaluation")
    }
}
