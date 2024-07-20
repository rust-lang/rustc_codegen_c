#![feature(rustc_private)]

extern crate parking_lot;
extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_const_eval;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_type_ir;
extern crate sharded_slab;
extern crate tracing;

use std::sync::Arc;

use rustc_ast::expand::allocator::AllocatorKind;
use rustc_codegen_ssa::back::link::link_binary;
use rustc_codegen_ssa::back::lto::{LtoModuleCodegen, SerializedModule, ThinModule};
use rustc_codegen_ssa::back::write::{
    CodegenContext, FatLtoInput, ModuleConfig, OngoingCodegen, TargetMachineFactoryFn,
};
use rustc_codegen_ssa::base::codegen_crate;
pub use rustc_codegen_ssa::traits::CodegenBackend;
use rustc_codegen_ssa::traits::{
    ExtraBackendMethods, ModuleBufferMethods, ThinBufferMethods, WriteBackendMethods,
};
use rustc_codegen_ssa::{CodegenResults, CompiledModule, ModuleCodegen};
use rustc_data_structures::fx::FxIndexMap;
use rustc_errors::{DiagCtxtHandle, FatalError};
use rustc_metadata::EncodedMetadata;
use rustc_middle::dep_graph::{WorkProduct, WorkProductId};
use rustc_middle::ty::TyCtxt;
use rustc_middle::util::Providers;
use rustc_session::config::{OptLevel, OutputFilenames};
use rustc_session::Session;
use rustc_span::ErrorGuaranteed;

use crate::module::ModuleContext;

mod archive;
mod base;
mod builder;
mod context;
mod module;
mod utils;
mod write;

rustc_fluent_macro::fluent_messages! { "../messages.ftl" }

#[derive(Clone)]
pub struct CCodegen {}

impl CodegenBackend for CCodegen {
    fn locale_resource(&self) -> &'static str {
        crate::DEFAULT_LOCALE_RESOURCE
    }

    fn provide(&self, providers: &mut Providers) {
        providers.global_backend_features = |_tcx, ()| vec![]
    }

    fn codegen_crate(
        &self,
        tcx: TyCtxt<'_>,
        metadata: EncodedMetadata,
        need_metadata_module: bool,
    ) -> Box<dyn std::any::Any> {
        let target_cpu = match tcx.sess.opts.cg.target_cpu {
            Some(ref name) => name,
            None => tcx.sess.target.cpu.as_ref(),
        }
        .to_owned();

        let ongoing_codegen =
            codegen_crate(self.clone(), tcx, target_cpu, metadata, need_metadata_module);
        Box::new(ongoing_codegen)
    }

    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn std::any::Any>,
        sess: &Session,
        _outputs: &OutputFilenames,
    ) -> (CodegenResults, FxIndexMap<WorkProductId, WorkProduct>) {
        ongoing_codegen.downcast::<OngoingCodegen<Self>>().expect("expected CCodegen").join(sess)
    }

    fn link(
        &self,
        sess: &Session,
        codegen_results: CodegenResults,
        outputs: &OutputFilenames,
    ) -> Result<(), ErrorGuaranteed> {
        link_binary(sess, &crate::archive::ArArchiveBuilderBuilder, &codegen_results, outputs)
    }

    fn supports_parallel(&self) -> bool {
        false // Maybe true?
    }
}

impl ExtraBackendMethods for CCodegen {
    fn codegen_allocator(
        &self,
        _tcx: TyCtxt<'_>,
        _module_name: &str,
        _kind: AllocatorKind,
        _alloc_error_handler_kind: AllocatorKind,
    ) -> Self::Module {
        todo!()
    }

    fn compile_codegen_unit(
        &self,
        tcx: TyCtxt<'_>,
        cgu_name: rustc_span::Symbol,
    ) -> (ModuleCodegen<Self::Module>, u64) {
        base::compile_codegen_unit(tcx, cgu_name)
    }

    fn target_machine_factory(
        &self,
        _sess: &Session,
        _opt_level: OptLevel,
        _target_features: &[String],
    ) -> TargetMachineFactoryFn<Self> {
        Arc::new(|_| Ok(()))
    }
}

pub struct ModuleBuffer;

impl ModuleBufferMethods for ModuleBuffer {
    fn data(&self) -> &[u8] {
        unimplemented!()
    }
}

pub struct ThinBuffer;

impl ThinBufferMethods for ThinBuffer {
    fn data(&self) -> &[u8] {
        unimplemented!()
    }

    fn thin_link_data(&self) -> &[u8] {
        unimplemented!()
    }
}

impl WriteBackendMethods for CCodegen {
    type Module = ModuleContext;
    type TargetMachine = ();
    type TargetMachineError = ();
    type ModuleBuffer = ModuleBuffer;
    type ThinData = ();
    type ThinBuffer = ThinBuffer;

    fn run_link(
        cgcx: &CodegenContext<Self>,
        dcx: DiagCtxtHandle<'_>,
        modules: Vec<ModuleCodegen<Self::Module>>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        write::link(cgcx, dcx, modules)
    }

    fn run_fat_lto(
        _cgcx: &CodegenContext<Self>,
        _modules: Vec<FatLtoInput<Self>>,
        _cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<LtoModuleCodegen<Self>, FatalError> {
        unimplemented!()
    }

    fn run_thin_lto(
        _cgcx: &CodegenContext<Self>,
        _modules: Vec<(String, Self::ThinBuffer)>,
        _cached_modules: Vec<(
            SerializedModule<Self::ModuleBuffer>,
            rustc_middle::dep_graph::WorkProduct,
        )>,
    ) -> Result<(Vec<LtoModuleCodegen<Self>>, Vec<WorkProduct>), rustc_errors::FatalError> {
        unimplemented!()
    }

    fn print_pass_timings(&self) {
        unimplemented!()
    }

    fn print_statistics(&self) {
        unimplemented!()
    }

    unsafe fn optimize(
        _cgcx: &CodegenContext<Self>,
        _dcx: DiagCtxtHandle<'_>,
        _module: &ModuleCodegen<Self::Module>,
        _config: &ModuleConfig,
    ) -> Result<(), FatalError> {
        Ok(())
    }

    fn optimize_fat(
        _cgcx: &CodegenContext<Self>,
        _llmod: &mut ModuleCodegen<Self::Module>,
    ) -> Result<(), FatalError> {
        unimplemented!()
    }

    unsafe fn optimize_thin(
        _cgcx: &CodegenContext<Self>,
        _thin: ThinModule<Self>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        unimplemented!()
    }

    unsafe fn codegen(
        cgcx: &CodegenContext<Self>,
        dcx: DiagCtxtHandle<'_>,
        module: ModuleCodegen<Self::Module>,
        config: &ModuleConfig,
    ) -> Result<CompiledModule, FatalError> {
        write::codegen(cgcx, dcx, module, config)
    }

    fn prepare_thin(
        _module: ModuleCodegen<Self::Module>,
        _want_summary: bool,
    ) -> (String, Self::ThinBuffer) {
        unimplemented!()
    }

    fn serialize_module(
        _module: rustc_codegen_ssa::ModuleCodegen<Self::Module>,
    ) -> (String, Self::ModuleBuffer) {
        unimplemented!()
    }
}

/// This is the entrypoint for a hot plugged codegen backend.
#[no_mangle]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(CCodegen {})
}
