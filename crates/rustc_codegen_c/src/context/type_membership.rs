use rustc_codegen_ssa::traits::TypeMembershipMethods;

use crate::context::CodegenCx;

impl<'tcx, 'mx> TypeMembershipMethods<'tcx> for CodegenCx<'tcx, 'mx> {}
