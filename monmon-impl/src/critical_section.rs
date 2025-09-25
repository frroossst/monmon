/// A macro to denote critical sections in the code.
/// This macro essentially acts as a marker and does
/// not enforce any synchronization itself.
/// ```rust
/// # use monmon_impl::critical_section;
/// let mut x = 0;
/// critical_section!({
///     x += 1;
/// });
/// ```
#[macro_export]
macro_rules! critical_section {
    ($code:block) => {
        $code
    };
}
