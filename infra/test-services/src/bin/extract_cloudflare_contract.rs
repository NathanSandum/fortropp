use std::error::Error;
use test_services::openapi_contract::extract;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let (Some(spec_path), Some(commit), Some(out_path)) = (args.next(), args.next(), args.next())
    else {
        return Err("usage: extract_cloudflare_contract <openapi.json> <commit> <out.json>".into());
    };
    let spec: serde_json::Value = serde_json::from_slice(&std::fs::read(spec_path)?)?;
    let contract = extract(&spec, &commit)?;
    let mut rendered = serde_json::to_string_pretty(&contract)?;
    rendered.push('\n');
    std::fs::write(out_path, rendered)?;
    Ok(())
}
