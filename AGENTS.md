# Agent Instructions

- Rust builds rely on aggressive compiler profiles defined in `.cargo/config.toml` (opt-level 3, LTO, single codegen unit, panic=abort). This keeps runtime performance high at the cost of noticeably longer compile times—please leave these settings in place unless you have explicit approval to change them.
