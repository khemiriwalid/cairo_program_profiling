use std::error::Error;

use cairo_vm::cairo_run::{CairoRunConfig, cairo_run};
use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;


fn read_trace_csv(file_path: &str) -> Result<Vec<(usize, usize, usize)>, Box<dyn Error>> {
    // Open the CSV file
    let mut reader = csv::Reader::from_path(file_path)?;
    
    let mut trace_data = Vec::new();

    for result in reader.records() {
        let record = result?; // Get the current record
        
        let pc: usize = record[0].parse()?;
        let ap: usize = record[1].parse()?;
        let fp: usize = record[2].parse()?;

        trace_data.push((pc, ap, fp));
    }

    Ok(trace_data)
}

fn run_program(
    data: &[u8],
    proof_mode: bool,
    layout: Option<LayoutName>,
    trace: Option<&[(usize, usize, usize)]>,
    error: Option<&str>,
) {
    let mut hint_executor = BuiltinHintProcessor::new_empty();
    let cairo_run_config: CairoRunConfig<'_> = CairoRunConfig { 
        entrypoint: "__main__", 
        layout: layout.unwrap_or(LayoutName::all_cairo),
        relocate_mem: true,
        trace_enabled: true,
        proof_mode,
        ..Default::default()
    };
    println!("Initialized Cairo VM!");
    let res = cairo_run(data, &cairo_run_config, &mut hint_executor);
    if let Some(error) = error {
        assert!(res.is_err());
        assert!(res.err().unwrap().to_string().contains(error));
        return;
    }
    let runner = res.expect("Execution failed");
    if let Some(trace) = trace {
        let expected_trace: Vec<_> = trace
            .iter()
            .copied()
            .map(|(pc, ap, fp)| RelocatedTraceEntry { pc, ap, fp })
            .collect();
        let trace = runner.relocated_trace.as_ref().unwrap();
        assert_eq!(trace.len(), expected_trace.len());
        for (entry, expected) in trace.iter().zip(expected_trace.iter()) {
            assert_eq!(entry, expected);
        }
    }
}


fn main() {

    let program_data = include_bytes!("program.json");
    let file_path = "src/trace.csv"; // Path to your CSV file
    let trace_data = read_trace_csv(file_path).unwrap();
    run_program(program_data.as_slice(), false, None, Some(&trace_data), None)
}

