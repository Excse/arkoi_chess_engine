use std::fmt::Write;

use reedline::ExternalPrinter;

pub struct WrappedPrinter(ExternalPrinter<String>);

impl Write for WrappedPrinter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        // TODO: Remove this unwrap
        self.0.print(s.to_string()).unwrap();
        Ok(())
    }
}
