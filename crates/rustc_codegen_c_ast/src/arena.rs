use crate::decl::CDeclKind;
use crate::expr::CExprKind;
use crate::func::CFuncKind;
use crate::r#type::CTyKind;
use crate::stmt::CStmtKind;

rustc_arena::declare_arena!([
    [] decl: CDeclKind<'tcx>,
    [] expr: CExprKind<'tcx>,
    [] func: CFuncKind<'tcx>,
    [] stmt: CStmtKind<'tcx>,
    [] ty: CTyKind<'tcx>,
]);
