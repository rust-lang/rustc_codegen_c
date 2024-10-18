//! This module defines the memory arena for C AST nodes.

use crate::decl::CDeclKind;
use crate::expr::CExprKind;
use crate::func::CFuncKind;
use crate::stmt::CStmtKind;
use crate::ty::CTyKind;

/// Memory arena for C AST nodes.
rustc_arena::declare_arena!([
    [] decl: CDeclKind<'tcx>,
    [] expr: CExprKind<'tcx>,
    [] func: CFuncKind<'tcx>,
    [] stmt: CStmtKind<'tcx>,
    [] ty: CTyKind<'tcx>,
]);
