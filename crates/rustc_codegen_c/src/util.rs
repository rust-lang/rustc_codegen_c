use rustc_middle::ty::TyCtxt;

pub fn global_backend_features(_tcx: TyCtxt<'_>) -> Vec<String> {
    vec![]
}
