use anyhow::Result;
use serenity::all::{Context, CreateEmbed, CreateMessage, Message};
use std::sync::Arc;
use crate::database::Database;

pub async fn execute(ctx: &Context, msg: &Message, _db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        let embed = crate::utils::create_error_embed(
            "Missing Code",
            "Usage: `!toast <rust_code>`\nExample: `!toast let x = 5;`\n\nProvide Rust code to analyze."
        );
        msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let code = args.join(" ");

    // Rust-specific analysis
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    let mut context_info = Vec::new();

    // Critical Rust errors
    if code.matches('{').count() != code.matches('}').count() {
        errors.push("[ERROR] Unbalanced braces - Expected equal number of '{' and '}'");
    }

    if code.matches('(').count() != code.matches(')').count() {
        errors.push("[ERROR] Unbalanced parentheses - Expected equal number of '(' and ')'");
    }

    if code.matches('[').count() != code.matches(']').count() {
        errors.push("[ERROR] Unbalanced brackets - Expected equal number of '[' and ']'");
    }

    if code.contains("= =") {
        errors.push("[ERROR] Syntax error - Found '= =' instead of '=='");
    }

    if code.contains(".await") && !code.contains("async") {
        errors.push("[ERROR] '.await' requires an async context - Add 'async' keyword to function");
    }

    // Rust-specific warnings
    if code.contains("unwrap()") {
        warnings.push("[WARNING] '.unwrap()' will panic on None/Err - Consider using '?' operator or 'expect()'");
        context_info.push("Better: result? or result.expect(\"descriptive message\")");
    }

    if code.contains("expect(") && code.contains("unwrap()") {
        suggestions.push("[SUGGESTION] Consistent error handling - Choose either expect() or ? throughout");
    }

    if code.contains("panic!") {
        warnings.push("[WARNING] 'panic!' will crash the program - Consider returning Result<T, E>");
        context_info.push("Better: Return Err(anyhow!(\"error message\")) instead of panicking");
    }

    if code.contains("todo!()") || code.contains("unimplemented!()") {
        warnings.push("[WARNING] Placeholder macro found - Implementation incomplete");
    }

    if code.contains("unsafe") {
        warnings.push("[WARNING] UNSAFE block detected - Requires careful review for memory safety");
        context_info.push("Unsafe code bypasses Rust's safety guarantees - ensure soundness");
    }

    // Ownership and borrowing
    if code.contains("clone()") {
        suggestions.push("[SUGGESTION] '.clone()' creates heap copy - Consider using references (&T) instead");
        context_info.push("Cloning can be expensive - use '&' for borrowing when possible");
    }

    if code.contains("&mut") && code.split("&mut").count() > 2 {
        warnings.push("[WARNING] Multiple mutable borrows detected - May violate borrowing rules");
        context_info.push("Rust allows only one mutable borrow at a time per scope");
    }

    if code.contains("Rc::") {
        suggestions.push("[SUGGESTION] Using Rc<T> - Consider if Arc<T> needed for thread safety");
        context_info.push("Rc = single-threaded, Arc = thread-safe (atomic reference counting)");
    }

    if code.contains("RefCell") || code.contains("Cell") {
        suggestions.push("[SUGGESTION] Interior mutability pattern detected - Ensure borrow rules at runtime");
        context_info.push("RefCell panics on borrow rule violations at runtime, not compile time");
    }

    // Memory and performance
    if code.contains("Box::new") {
        suggestions.push("[SUGGESTION] 'Box<T>' allocates on heap - Ensure heap allocation is necessary");
        context_info.push("Box is used for heap allocation, recursive types, or trait objects");
    }

    if code.contains("Vec::new()") && code.contains("push(") {
        suggestions.push("[SUGGESTION] Vec growth - Use 'Vec::with_capacity()' if size is known");
        context_info.push("Pre-allocating capacity avoids multiple reallocations");
    }

    if code.contains("String::new()") && code.contains("push_str") {
        suggestions.push("[SUGGESTION] String growth - Consider 'String::with_capacity()' for efficiency");
    }

    // Async/await patterns
    if code.contains(".await") {
        context_info.push("Context: .await suspends execution until future completes");
    }

    if code.contains("tokio::spawn") || code.contains("async move") {
        context_info.push("Context: Spawned tasks require 'static lifetime or moved ownership");
    }

    // Error handling patterns
    if code.contains("?") {
        context_info.push("Context: '?' operator propagates errors to caller (requires Result return)");
    }

    if code.contains("Result<") && !code.contains("?") && !code.contains("unwrap") {
        suggestions.push("[SUGGESTION] Result type used - Consider using '?' for cleaner error propagation");
    }

    if code.contains("Option<") && code.contains("unwrap()") {
        warnings.push("[WARNING] Unwrapping Option - Use 'if let', 'match', or '?' instead");
        context_info.push("Better: if let Some(val) = option { ... } or option?");
    }

    // Lifetime annotations
    if code.contains("'static") {
        context_info.push("Context: 'static lifetime means data lives for entire program duration");
    }

    if code.matches('\'').count() > 0 && (code.contains("<'") || code.contains("&'")) {
        context_info.push("Context: Lifetime annotations ensure references remain valid");
    }

    // Type system
    if code.contains("as ") {
        warnings.push("[WARNING] Type casting with 'as' - Ensure conversion is safe and intended");
        context_info.push("'as' performs potentially lossy conversions - consider TryFrom/Into");
    }

    if code.contains("dyn ") {
        context_info.push("Context: 'dyn Trait' creates trait object (dynamic dispatch at runtime)");
    }

    if code.contains("impl ") && code.contains("for ") {
        context_info.push("Context: 'impl Trait for Type' implements trait methods for a type");
    }

    // Concurrency
    if code.contains("Mutex") || code.contains("RwLock") {
        suggestions.push("[SUGGESTION] Synchronization primitive detected - Ensure minimal lock contention");
        context_info.push("Mutex = mutual exclusion, RwLock = multiple readers or single writer");
    }

    if code.contains("Arc<Mutex") {
        context_info.push("Context: Arc<Mutex<T>> pattern for shared mutable state across threads");
    }

    if code.contains("Send") || code.contains("Sync") {
        context_info.push("Context: Send = can transfer between threads, Sync = can share references");
    }

    // Common patterns
    if code.contains("match ") {
        context_info.push("Context: 'match' provides exhaustive pattern matching - all cases handled");
    }

    if code.contains("if let ") {
        context_info.push("Context: 'if let' for simple pattern matching without full match");
    }

    // Metrics
    let line_count = code.lines().count();
    let char_count = code.len();
    let word_count = code.split_whitespace().count();

    // Build embed
    let mut embed = CreateEmbed::new()
        .title("Rust Code Analysis - Toast")
        .description(format!("```rust\n{}\n```", code))
        .color(if errors.is_empty() { 0x5865F2 } else { 0xED4245 })
        .field(
            "Code Metrics",
            format!(
                "**Language:** Rust\n\
                **Lines:** {}\n\
                **Characters:** {}\n\
                **Tokens:** {}",
                line_count, char_count, word_count
            ),
            false
        );

    if !errors.is_empty() {
        embed = embed.field(
            "Critical Errors",
            errors.join("\n"),
            false
        );
    }

    if !warnings.is_empty() {
        embed = embed.field(
            "Warnings",
            warnings.join("\n"),
            false
        );
    }

    if !suggestions.is_empty() {
        embed = embed.field(
            "Suggestions",
            suggestions.join("\n"),
            false
        );
    }

    if !context_info.is_empty() {
        let context_text = context_info.join("\n");
        if context_text.len() > 1024 {
            let truncated = format!("{}...", &context_text[..1020]);
            embed = embed.field("Rust Context", truncated, false);
        } else {
            embed = embed.field("Rust Context", context_text, false);
        }
    }

    if errors.is_empty() && warnings.is_empty() && suggestions.is_empty() {
        embed = embed.field(
            "Status",
            "No obvious issues detected - Code looks clean at first glance",
            false
        );
    }

    embed = embed
        .footer(serenity::all::CreateEmbedFooter::new("Toast - Rust Code Analyzer"))
        .timestamp(serenity::all::Timestamp::now());

    msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(embed)).await?;

    Ok(())
}
