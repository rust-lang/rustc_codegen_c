use rustc_ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_codegen_ssa::traits::{AsmMethods, GlobalAsmOperandRef};

use crate::context::CodegenCx;

impl<'tcx, 'mx> AsmMethods<'tcx> for CodegenCx<'tcx, 'mx> {
    fn codegen_global_asm(
        &self,
        template: &[InlineAsmTemplatePiece],
        operands: &[GlobalAsmOperandRef<'tcx>],
        options: InlineAsmOptions,
        line_spans: &[rustc_span::Span],
    ) {
        todo!()
    }
}
