use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 || !["plan", "run"].contains(&args[1].as_str()) {
        return Err(
            "usage: pvrpc-pir-scale plan|run ROWS WIDTH MEMORY_BUDGET_BYTES QUERY_COUNT".into(),
        );
    }
    let rows = args[2].parse()?;
    let width = args[3].parse()?;
    let budget = args[4].parse()?;
    let count = args[5].parse()?;
    let result = if args[1] == "plan" {
        pvrpc_pir_scale::plan(rows, width, budget)?
    } else {
        pvrpc_pir_scale::run(rows, width, budget, count)?
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
