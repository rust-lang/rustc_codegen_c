use rustc_codegen_ssa::traits::TypeMembershipMethods;

use crate::context::CodegenCx;

impl<'tcx> TypeMembershipMethods<'tcx> for CodegenCx<'tcx> {}
