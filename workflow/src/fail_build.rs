pub trait UnwrapOrFailBuild<V> {
    fn unwrap_or_fail_build(self) -> V;
}

impl<V, E> UnwrapOrFailBuild<V> for Result<V, E>
where
    E: std::fmt::Display,
{
    fn unwrap_or_fail_build(self) -> V {
        self.unwrap_or_else(|err| fail_build(err.to_string()))
    }
}

pub fn fail_build<S: AsRef<str>>(error: S) -> ! {
    eprintln!("-- BUILD FAILED: {}", error.as_ref());
    std::process::exit(1);
}
