use std::fs;
use svg_layout_checker::check_layout;

fn main() -> anyhow::Result<()> {
    let dir = "/home/noah/src/rust-de-specialization/coursera-assets/c12-graph-algorithms";
    let entries: Vec<_> = fs::read_dir(dir)?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "svg").unwrap_or(false))
        .collect();
    let mut all_pass = true;
    for e in entries {
        let path = e.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let svg = fs::read_to_string(&path)?;
        let report = check_layout(&svg);
        if report.all_pass() {
            println!("[PASS] {}", name);
        } else {
            all_pass = false;
            println!("[FAIL] {}", name);
            for f in report.failures() {
                println!("       {}", f);
            }
        }
    }
    if !all_pass {
        anyhow::bail!("at least one master failed the layout contract");
    }
    Ok(())
}
