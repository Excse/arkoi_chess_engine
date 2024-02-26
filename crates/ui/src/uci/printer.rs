use std::fmt::Write;

use reedline::ExternalPrinter;

pub struct Printer(ExternalPrinter<String>);

impl Printer {
    pub fn new(printer: ExternalPrinter<String>) -> Self {
        Self(printer)
    }
}

impl Write for Printer {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        // TODO: Remove unwrap
        self.0.print(s.to_string()).unwrap();
        Ok(())
    }
}
