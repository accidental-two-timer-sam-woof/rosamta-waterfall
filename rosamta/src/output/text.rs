use std::io::{self, Write};

use rosamta_core::diag::Diagnostics;

pub(crate) fn write(stderr: &mut impl Write, diagnostics: &Diagnostics) -> io::Result<()> {
    for diagnostic in diagnostics.iter() {
        writeln!(
            stderr,
            "{}[{}]: {}",
            diagnostic.severity.as_str(),
            diagnostic.code.as_str(),
            diagnostic.message,
        )?;
    }
    Ok(())
}
