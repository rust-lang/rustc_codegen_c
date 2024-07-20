use std::fs;
use std::io::Write;
use std::process::Stdio;

use rustc_codegen_ssa::back::command::Command;
use rustc_codegen_ssa::back::write::{CodegenContext, ModuleConfig};
use rustc_codegen_ssa::{CompiledModule, ModuleCodegen};
use rustc_errors::{DiagCtxtHandle, FatalError};
use rustc_session::config::OutputType;
use tracing::error;

use crate::module::ModuleContext;

pub(crate) unsafe fn codegen(
    cgcx: &CodegenContext<crate::CCodegen>,
    _dcx: DiagCtxtHandle<'_>,
    module: ModuleCodegen<ModuleContext>,
    _config: &ModuleConfig,
) -> Result<CompiledModule, FatalError> {
    let module_name = module.name.clone();
    let module_name = Some(&module_name[..]);
    let obj_out = cgcx.output_filenames.temp_path(OutputType::Object, module_name);
    let c_out = obj_out.with_extension("c");

    // output c source code
    let c_out_file = fs::File::create(&c_out).map_err(|_| FatalError)?;
    writeln!(&c_out_file, "// file: {}.c", module.name).map_err(|_| FatalError)?;
    write!(&c_out_file, "{}", module.module_llvm).map_err(|_| FatalError)?;

    // invoke cc to compile
    // TODO: configure cc
    // TODO: handle long command line (windows)
    // TODO: flush_linked_file (windows)
    let mut cmd = Command::new("clang");
    cmd.arg(&c_out).arg("-o").arg(&obj_out).arg("-c");
    let mut cmd = cmd.command();
    let output = match cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|child| child.wait_with_output())
    {
        Ok(output) => {
            output
            // flush_linked_file(&output, out_filename)?;
        }
        Err(e) => {
            error!("failed to spawn C compiler: {}", e);
            return Err(FatalError);
        }
    };

    if !output.status.success() {
        error!("compiler stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        error!("compiler stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        return Err(FatalError);
    }

    Ok(module.into_compiled_module(true, false, false, false, false, &cgcx.output_filenames))
}

pub(crate) fn link(
    _cgcx: &CodegenContext<crate::CCodegen>,
    _dcx: DiagCtxtHandle<'_>,
    mut _modules: Vec<ModuleCodegen<ModuleContext>>,
) -> Result<ModuleCodegen<ModuleContext>, FatalError> {
    unimplemented!();
}
