#![no_main]

use garnet_parser::{parse_source_with_budget, ParseBudget};
use libfuzzer_sys::fuzz_target;

const MAX_FUZZ_SOURCE_BYTES: usize = 64 * 1024;

fn fuzz_budget() -> ParseBudget {
    ParseBudget {
        max_source_bytes: MAX_FUZZ_SOURCE_BYTES,
        max_tokens: 16 * 1024,
        max_depth: 128,
        max_literal_bytes: 8 * 1024,
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_FUZZ_SOURCE_BYTES {
        return;
    }

    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    let _ = parse_source_with_budget(source, fuzz_budget());
});
