/// A macro to denote critical sections in the code.
/// This macro essentially acts as a marker and does
/// not enforce any synchronization itself.
#[macro_export]
macro_rules! critical_section {
    ($code:block) => {
        $code
    };
}
