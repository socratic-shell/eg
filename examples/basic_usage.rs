//! Basic usage example for the eg library

use eg::Eg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Search for examples in a popular crate
    println!("Searching for serde examples...");
    
    let result = Eg::rust_crate("serde")
        .search()
        .await?;
    
    println!("Found {} examples in serde v{}", 
             result.total_examples, result.version);
    
    // Search with a pattern
    println!("\nSearching for 'derive' in tokio examples...");
    
    let result = Eg::rust_crate("tokio")
        .pattern(r"derive")?
        .search()
        .await?;
    
    println!("Found {} examples, {} matched the pattern", 
             result.total_examples, result.matched_examples);
    
    for example in result.examples.iter().take(3) {
        match example {
            eg::Example::ExampleOnDisk { path, search_matches } => {
                println!("  📁 {} ({} matches)", path.display(), search_matches.len());
            }
            eg::Example::ExampleInMemory { filename, search_matches, .. } => {
                println!("  💾 {} ({} matches)", filename, search_matches.len());
            }
        }
    }
    
    Ok(())
}
