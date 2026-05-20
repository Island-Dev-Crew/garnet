#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(src) = std::str::from_utf8(data) {
        // Use a strict parsing budget to limit CPU/memory/recursion during aggressive fuzzing
        let budget = garnet_parser::ParseBudget {
            max_source_bytes: 4096, 
            max_tokens: 1024,
            max_depth: 32,
            max_literal_bytes: 512,
        };
        let _ = garnet_parser::parse_source_with_budget(src, budget);
    }
});
