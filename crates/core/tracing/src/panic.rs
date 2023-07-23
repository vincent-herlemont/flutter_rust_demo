// use std::backtrace::Backtrace;
use std::panic::{self};
use std::sync::Once;
use tracing_original::error;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        let next = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let backtrace = backtrace::Backtrace::new();
            error!("panic: {},\n{:?}\n", info, backtrace);
            next(info);
        }));
    });
}
