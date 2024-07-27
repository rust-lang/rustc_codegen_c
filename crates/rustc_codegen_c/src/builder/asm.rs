use rustc_ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_codegen_ssa::traits::{AsmBuilderMethods, InlineAsmOperandRef};
use rustc_middle::ty::Instance;

use crate::builder::Builder;

impl<'tcx, 'mx> AsmBuilderMethods<'tcx> for Builder<'_, 'tcx, 'mx> {
    fn codegen_inline_asm(
        &mut self,
        template: &[InlineAsmTemplatePiece],
        operands: &[InlineAsmOperandRef<'tcx, Self>],
        options: InlineAsmOptions,
        line_spans: &[rustc_span::Span],
        instance: Instance<'_>,
        dest: Option<Self::BasicBlock>,
        catch_funclet: Option<(Self::BasicBlock, Option<&Self::Funclet>)>,
    ) {
        todo!()
    }
}
