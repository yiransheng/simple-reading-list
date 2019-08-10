#[doc(hidden)]
#[macro_export]
macro_rules! loop_panic_when_stuck {
    ($b: block) => {{
        let mut loop_count_before_panic: usize = 0;
        loop {
            if loop_count_before_panic > 100_000 {
                log::error!("Infinite loop detected, panic now...");
                panic!()
            }
            loop_count_before_panic += 1;
            $b
        }
    }};
}
