//! Basic usage example for the eg library

use eg::Eg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Search for examples in a popular crate
    println!("Searching for serde examples...");
    
    let result = Eg::rust_crate("serde")
        .search()
        .await?;
    
    println!("Crate extracted to: {}", result.checkout_path.display());
    println!("Found {} example matches, {} other matches", 
             result.example_matches.len(), result.other_matches.len());
    
    // Search with a pattern
    println!("\nSearching for 'derive' in tokio examples...");
    
    let result = Eg::rust_crate("tokio")
        .pattern(r"derive")?
        .context_lines(3)
        .search()
        .await?;
    
    println!("Crate extracted to: {}", result.checkout_path.display());
    println!("Found {} example matches, {} other matches", 
             result.example_matches.len(), result.other_matches.len());
    
    // Show first few matches
    for (i, m) in result.example_matches.iter().take(3).enumerate() {
        println!("\n--- Example Match {} ---", i + 1);
        println!("File: {}", m.file_path.display());
        println!("Line {}: {}", m.line_number, m.line_content);
        if !m.context_before.is_empty() {
            println!("Context before: {:?}", m.context_before);
        }
        if !m.context_after.is_empty() {
            println!("Context after: {:?}", m.context_after);
        }
    }
    
    Ok(())
}
