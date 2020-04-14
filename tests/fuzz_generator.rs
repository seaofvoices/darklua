use darklua_core::{nodes::Block, LuaGenerator, ToLua};
use std::time::{Duration, Instant};

mod fuzz;
mod utils;

use fuzz::*;

macro_rules! fuzz_test {
    ($context:expr, $column_span:expr) => {
        let block = Block::fuzz(&mut $context);

        let mut generator = LuaGenerator::new($column_span);
        block.to_lua(&mut generator);
        let lua_code = generator.into_string();

        let generated_block = match utils::try_parse_input(&lua_code) {
            Ok(block) => block,
            Err(error) => panic!(
                concat!(
                    "could not parse content: {:?}\n",
                    "============================================================\n",
                    ">>> Lua code input:\n{}\n",
                    "============================================================\n",
                    "\n",
                    "============================================================\n",
                    ">>> Block that produced the generated code:\n{:?}\n",
                    "============================================================\n",
                ),
                error,
                lua_code,
                block,
            ),
        };

        assert_eq!(
            block,
            generated_block,
            concat!(
                "\n",
                "============================================================\n",
                ">>> Block generated from node fuzz:\n{:?}\n",
                ">>> Lua code generated:\n{}\n",
                "============================================================\n",
                "\n",
                "============================================================\n",
                ">>> Block generated from parsed generated code:\n{:?}\n",
                ">>> Lua code generated:\n{}\n",
                "============================================================\n",
            ),
            block,
            lua_code,
            generated_block,
            generated_block.to_lua_string(),
        );
    };
    ($context:expr) => {
        fuzz_test!($context, 80);
    };
}

fn run_for_minimum_time<F: Fn()>(func: F) {
    let duration = get_fuzz_duration();
    let start = Instant::now();

    loop {
        func();

        if Instant::now().duration_since(start) > duration {
            break
        }
    }
}

fn get_fuzz_duration() -> Duration {
    let millis = option_env!("FUZZ_DURATION_MILLISECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1500);

    Duration::from_millis(millis)
}

#[test]
fn fuzz_single_statement() {
    run_for_minimum_time(|| {
        fuzz_test!(FuzzContext::new(1, 5));
    });
}

#[test]
fn fuzz_small_block() {
    run_for_minimum_time(|| {
        fuzz_test!(FuzzContext::new(20, 40));
    });
}

#[test]
fn fuzz_medium_block() {
    run_for_minimum_time(|| {
        fuzz_test!(FuzzContext::new(100, 200));
    });
}

#[test]
fn fuzz_large_block() {
    run_for_minimum_time(|| {
        fuzz_test!(FuzzContext::new(200, 500));
    });
}

#[test]
fn fuzz_column_span() {
    run_for_minimum_time(|| {
        for i in 0..80 {
            fuzz_test!(FuzzContext::new(20, 40), i);
        }
    });
}
