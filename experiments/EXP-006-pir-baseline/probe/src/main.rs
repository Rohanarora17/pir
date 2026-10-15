use std::{error::Error, path::Path};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        return Err(
            "usage: pvrpc-pir-baseline CORPUS_DIRECTORY simple|frodo index|keyword 3072|4096"
                .into(),
        );
    }
    let value = pvrpc_pir_baseline::experiment::run(
        Path::new(&args[1]),
        &args[2],
        &args[3],
        args[4].parse()?,
    )?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
