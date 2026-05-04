use std::fs;
use svg_layout_checker::check_layout;
fn main() -> anyhow::Result<()> {
    let dir = "/home/noah/src/rust-de-specialization/coursera-assets/c12-graph-algorithms";
    let mut total = 0;
    let mut passed = 0;
    let mut failures = Vec::new();
    for e in fs::read_dir(dir)?.flatten() {
        let path = e.path();
        if path.extension().map(|x| x == "svg").unwrap_or(false) {
            total += 1;
            let svg = fs::read_to_string(&path)?;
            let report = check_layout(&svg);
            if report.all_pass() {
                passed += 1;
            } else {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let fails: Vec<String> =
                    report.failures().iter().map(|f| format!("{}", f)).collect();
                failures.push((name, fails));
            }
        }
    }
    println!("layout contract: {}/{} pass", passed, total);
    for (n, fs) in &failures {
        println!("  [FAIL] {}", n);
        for f in fs {
            println!("         {}", f);
        }
    }
    if !failures.is_empty() {
        anyhow::bail!("{} files failed", failures.len());
    }
    Ok(())
}
