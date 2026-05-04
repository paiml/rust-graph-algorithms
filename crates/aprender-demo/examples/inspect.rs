// Run the demo once and print the report so I can write tests with
// real values.
use aprender_demo::run_demo;

fn main() -> anyhow::Result<()> {
    let r = run_demo()?;
    println!("num_nodes:    {}", r.num_nodes);
    println!("bfs_order:    {:?}", r.bfs_order);
    println!("pagerank:     {:?}", r.pagerank);
    println!("pagerank_sum: {}", r.pagerank_sum);
    println!("sccs:         {:?}", r.sccs);
    println!("winner:       {}", r.winner);
    Ok(())
}
