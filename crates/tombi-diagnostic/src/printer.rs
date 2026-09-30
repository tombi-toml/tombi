mod pretty;
mod simple;

pub use pretty::Pretty;
pub use simple::Simple;

pub trait Print<Printer> {
    /// Writes the object to the writer in the format of the printer.
    fn print(&self, printer: &Printer, writer: &mut dyn std::io::Write) -> std::io::Result<()>;
}

impl<T, P> Print<P> for Vec<T>
where
    T: Print<P>,
{
    fn print(&self, printer: &P, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        for item in self {
            item.print(printer, writer)?;
        }
        Ok(())
    }
}
