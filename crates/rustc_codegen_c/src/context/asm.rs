use rustc_ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_codegen_ssa::traits::{AsmMethods, GlobalAsmOperandRef};

use crate::context::CodegenCx;

impl<'tcx> AsmMethods<'tcx> for CodegenCx<'tcx> {
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
