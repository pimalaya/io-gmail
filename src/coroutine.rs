//! Coroutine driver: the `GmailCoroutine` trait, its `GmailYield` /
//! `GmailCoroutineState`, and the `gmail_try!` macro (the coroutine
//! equivalent of `?`).

use alloc::vec::Vec;

#[derive(Debug)]
pub enum GmailCoroutineState<Y, R> {
    Yielded(Y),
    Complete(R),
}

pub trait GmailCoroutine {
    type Yield;
    type Return;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return>;
}

#[derive(Debug)]
pub enum GmailYield {
    WantsRead,
    WantsWrite(Vec<u8>),
}

#[macro_export]
macro_rules! gmail_try {
    ($coroutine:expr, $arg:expr $(,)?) => {
        match $crate::coroutine::GmailCoroutine::resume($coroutine, $arg) {
            $crate::coroutine::GmailCoroutineState::Yielded(y) => {
                return $crate::coroutine::GmailCoroutineState::Yielded(y.into());
            }
            $crate::coroutine::GmailCoroutineState::Complete(Err(err)) => {
                log::trace!("error during coroutine execution: {err}");
                return $crate::coroutine::GmailCoroutineState::Complete(Err(err.into()));
            }
            $crate::coroutine::GmailCoroutineState::Complete(Ok(value)) => value,
        }
    };
}
