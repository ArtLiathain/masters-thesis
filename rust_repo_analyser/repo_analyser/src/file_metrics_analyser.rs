use std::{error::Error, fs, path::PathBuf};

use csv::Writer;
use rust_code_analysis::{get_function_spaces, CodeMetrics};

#[derive(Debug, Clone, serde::Serialize)]
pub struct FlattenedMetrics {
    file_path: String,
    is_high_risk: bool,
    nargs_fn_nargs: f64,
    nargs_closure_nargs: f64,
    nargs_fn_nargs_sum: f64,
    nargs_closure_nargs_sum: f64,
    nargs_fn_nargs_avg: f64,
    nargs_closure_nargs_avg: f64,
    nargs_fn_nargs_min: f64,
    nargs_closure_nargs_min: f64,
    nargs_fn_nargs_max: f64,
    nargs_closure_nargs_max: f64,
    nexits_exit: f64,
    nexits_exit_sum: f64,
    nexits_exit_avg: f64,
    nexits_exit_min: f64,
    nexits_exit_max: f64,
    cognitive: f64,
    cognitive_sum: f64,
    cognitive_avg: f64,
    cognitive_min: f64,
    cognitive_max: f64,
    cyclomatic_cyclomatic: f64,
    cyclomatic_cyclomatic_sum: f64,
    cyclomatic_cyclomatic_avg: f64,
    cyclomatic_cyclomatic_min: f64,
    cyclomatic_cyclomatic_max: f64,
    halstead_u_operators: f64,
    halstead_operators: f64,
    halstead_u_operands: f64,
    halstead_operands: f64,
    halstead_volume: f64,
    halstead_difficulty: f64,
    halstead_level: f64,
    halstead_effort: f64,
    loc_sloc: f64,
    loc_ploc: f64,
    loc_cloc: f64,
    loc_lloc: f64,
    loc_blank: f64,
    mi_mi_original: f64,
    mi_mi_sei: f64,
    mi_mi_visual_studio: f64,
    abc_assignments: f64,
    abc_branches: f64,
    abc_conditions: f64,
    nom_functions: f64,
    nom_closures: f64,
    wmc_cyclomatic: f64,
}

pub fn flatten_metrics(
    file_path: &str,
    metrics: &CodeMetrics,
    is_high_risk: bool,
) -> FlattenedMetrics {
    FlattenedMetrics {
        file_path: file_path.to_string(),
        is_high_risk: is_high_risk,
        nargs_fn_nargs: metrics.nargs.fn_args(),
        nargs_closure_nargs: metrics.nargs.closure_args(),
        nargs_fn_nargs_sum: metrics.nargs.fn_args_sum(),
        nargs_closure_nargs_sum: metrics.nargs.closure_args_sum(),
        nargs_fn_nargs_avg: metrics.nargs.fn_args_average(),
        nargs_closure_nargs_avg: metrics.nargs.closure_args_average(),
        nargs_fn_nargs_min: metrics.nargs.fn_args_min(),
        nargs_closure_nargs_min: metrics.nargs.closure_args_min(),
        nargs_fn_nargs_max: metrics.nargs.fn_args_max(),
        nargs_closure_nargs_max: metrics.nargs.closure_args_max(),
        nexits_exit: metrics.nexits.exit(),
        nexits_exit_sum: metrics.nexits.exit_sum(),
        nexits_exit_avg: metrics.nexits.exit_average(),
        nexits_exit_min: metrics.nexits.exit_min(),
        nexits_exit_max: metrics.nexits.exit_max(),
        cognitive: metrics.cognitive.cognitive(),
        cognitive_sum: metrics.cognitive.cognitive_sum(),
        cognitive_avg: metrics.cognitive.cognitive_average(),
        cognitive_min: metrics.cognitive.cognitive_min(),
        cognitive_max: metrics.cognitive.cognitive_max(),
        cyclomatic_cyclomatic: metrics.cyclomatic.cyclomatic(),
        cyclomatic_cyclomatic_sum: metrics.cyclomatic.cyclomatic_sum(),
        cyclomatic_cyclomatic_avg: metrics.cyclomatic.cyclomatic_average(),
        cyclomatic_cyclomatic_min: metrics.cyclomatic.cyclomatic_min(),
        cyclomatic_cyclomatic_max: metrics.cyclomatic.cyclomatic_max(),
        halstead_u_operators: metrics.halstead.u_operators(),
        halstead_operators: metrics.halstead.operators(),
        halstead_u_operands: metrics.halstead.u_operands(),
        halstead_operands: metrics.halstead.operands(),
        halstead_volume: metrics.halstead.volume(),
        halstead_difficulty: metrics.halstead.difficulty(),
        halstead_level: metrics.halstead.level(),
        halstead_effort: metrics.halstead.effort(),
        loc_sloc: metrics.loc.sloc(),
        loc_ploc: metrics.loc.ploc(),
        loc_cloc: metrics.loc.cloc(),
        loc_lloc: metrics.loc.lloc(),
        loc_blank: metrics.loc.blank(),
        mi_mi_original: metrics.mi.mi_original(),
        mi_mi_sei: metrics.mi.mi_sei(),
        mi_mi_visual_studio: metrics.mi.mi_visual_studio(),
        abc_assignments: metrics.abc.assignments(),
        abc_branches: metrics.abc.branches(),
        abc_conditions: metrics.abc.conditions(),
        nom_functions: metrics.nom.functions(),
        nom_closures: metrics.nom.closures(),
        wmc_cyclomatic: metrics.wmc.total_wmc(),
    }
}

pub fn convert_files_to_metric_csv(
    file_path: String,
    output_file: String,
    is_high_risk: bool,
) -> Result<(), Box<dyn Error>> {
    let paths = fs::read_dir(file_path).unwrap();
    let mut wtr = Writer::from_writer(vec![]);
    for path in paths {
        let path = path?;

        println!("PATH: {}", path.path().display());
        let path_buf = PathBuf::from(path.path());
        let file_as_bytes = fs::read(&path_buf)?;
        let results = get_function_spaces(
            &rust_code_analysis::LANG::Cpp,
            file_as_bytes,
            &path_buf,
            None,
        )
        .unwrap();

        let flattened_metrics = flatten_metrics(
            &path.path().display().to_string().split("/").last().unwrap(),
            &results.metrics,
            is_high_risk,
        );
        wtr.serialize(flattened_metrics)?;
    }

    let data = String::from_utf8(wtr.into_inner()?)?;
    fs::write(output_file.clone(), data)
        .expect(&format!("Should be able to write to `{}`", output_file));

    Ok(())
}
