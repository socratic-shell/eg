//! Integration tests for the eg library

use eg::Eg;

/// Test searching a crate that's in our current project dependencies
/// Should use the version from cargo cache/src
#[tokio::test(flavor = "current_thread")]
async fn test_current_project_dependency() {
    // Use 'regex' since it's in our Cargo.toml
    let result = Eg::rust_crate("regex")
        .search()
        .await
        .expect("Should find regex crate");

    // Verify we got a result
    assert!(!result.version.is_empty(), "Should have a version");
    assert!(result.checkout_path.exists(), "Checkout path should exist");
    
    // Should come from cargo's cache (either src or our extraction cache)
    let cargo_home = home::cargo_home().expect("Should find cargo home");
    let is_from_cargo_src = result.checkout_path.starts_with(cargo_home.join("registry/src"));
    let is_from_our_cache = result.checkout_path.to_string_lossy().contains("eg/extractions");
    
    assert!(
        is_from_cargo_src || is_from_our_cache,
        "Should use cargo cache or our extraction cache, got: {}",
        result.checkout_path.display()
    );

    println!("✅ regex v{} found at: {}", result.version, result.checkout_path.display());
    
    // Verify the checkout contains expected Rust project structure
    assert!(result.checkout_path.join("Cargo.toml").exists(), "Should have Cargo.toml");
    assert!(result.checkout_path.join("src").exists(), "Should have src directory");
}

/// Test searching a crate that's NOT in our current project
/// Should use cargo cache if available, or download to our cache
#[tokio::test(flavor = "current_thread")]
async fn test_external_crate() {
    // Use 'uuid' - a well-known crate that's not in our dependencies
    let result = Eg::rust_crate("uuid")
        .search()
        .await
        .expect("Should find uuid crate");

    // Verify we got a result
    assert!(!result.version.is_empty(), "Should have a version");
    assert!(result.checkout_path.exists(), "Checkout path should exist");
    
    // Should be either in cargo's src cache OR our extraction cache
    let cargo_home = home::cargo_home().expect("Should find cargo home");
    let is_from_cargo_src = result.checkout_path.starts_with(cargo_home.join("registry/src"));
    let is_from_our_cache = result.checkout_path.to_string_lossy().contains("eg/extractions");
    
    assert!(
        is_from_cargo_src || is_from_our_cache,
        "Should be in cargo cache or our extraction cache, got: {}",
        result.checkout_path.display()
    );

    if is_from_cargo_src {
        println!("✅ uuid v{} found in cargo cache: {}", result.version, result.checkout_path.display());
    } else {
        println!("✅ uuid v{} downloaded to our cache: {}", result.version, result.checkout_path.display());
    }
    
    // Verify the checkout contains expected Rust project structure
    assert!(result.checkout_path.join("Cargo.toml").exists(), "Should have Cargo.toml");
    assert!(result.checkout_path.join("src").exists(), "Should have src directory");
}

/// Test pattern matching in examples
#[tokio::test(flavor = "current_thread")]
async fn test_pattern_matching() {
    // Use serde - very popular crate with good examples
    let result = Eg::rust_crate("serde")
        .pattern(r"derive")
        .expect("Should compile regex")
        .context_lines(2)
        .search()
        .await
        .expect("Should find serde crate");

    println!("✅ serde v{} search completed", result.version);
    println!("   Found {} example matches, {} other matches", 
             result.example_matches.len(), result.other_matches.len());

    // Should have found some matches (serde uses derive extensively)
    let total_matches = result.example_matches.len() + result.other_matches.len();
    assert!(total_matches > 0, "Should find some 'derive' matches in serde");

    // Verify match structure
    if let Some(first_match) = result.example_matches.first().or(result.other_matches.first()) {
        assert!(!first_match.file_path.as_os_str().is_empty(), "Should have file path");
        assert!(first_match.line_number > 0, "Should have valid line number");
        assert!(!first_match.line_content.is_empty(), "Should have line content");
        assert!(first_match.line_content.contains("derive"), "Line should contain 'derive'");
        
        println!("   Example match: {}:{} - {}", 
                 first_match.file_path.display(), 
                 first_match.line_number, 
                 first_match.line_content.trim());
    }
}

/// Test version constraint resolution
#[tokio::test(flavor = "current_thread")]
async fn test_version_constraints() {
    // Test explicit version constraint
    let result = Eg::rust_crate("serde")
        .version("^1.0")
        .search()
        .await
        .expect("Should find serde with version constraint");

    // Should find a 1.x version
    assert!(result.version.starts_with("1."), 
            "Should find 1.x version, got: {}", result.version);
    
    println!("✅ serde version constraint ^1.0 resolved to: {}", result.version);
}

/// Test error handling for non-existent crate
#[tokio::test(flavor = "current_thread")]
async fn test_nonexistent_crate() {
    let result = Eg::rust_crate("this-crate-definitely-does-not-exist-12345")
        .search()
        .await;

    assert!(result.is_err(), "Should fail for non-existent crate");
    
    let error = result.unwrap_err();
    println!("✅ Correctly failed for non-existent crate: {}", error);
}

/// Test that checkout paths are reused (caching works)
#[tokio::test(flavor = "current_thread")]
async fn test_caching() {
    // Search the same crate twice
    let result1 = Eg::rust_crate("uuid")
        .search()
        .await
        .expect("First search should succeed");

    let result2 = Eg::rust_crate("uuid")
        .search()
        .await
        .expect("Second search should succeed");

    // Should get the same version and path (cached)
    assert_eq!(result1.version, result2.version, "Should get same version");
    assert_eq!(result1.checkout_path, result2.checkout_path, "Should reuse same checkout path");
    
    println!("✅ Caching works: both searches used {}", result1.checkout_path.display());
}
