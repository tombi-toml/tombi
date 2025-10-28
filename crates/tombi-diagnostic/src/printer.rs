mod pretty;
mod simple;

pub use pretty::Pretty;
pub use simple::Simple;

pub trait Print<Printer> {
    /// Formats the object using the given formatter.
    fn print(&self, printer: &mut Printer, use_ansi_color: bool);
}

impl<T, P> Print<P> for Vec<T>
where
    T: Print<P>,
{
    fn print(&self, printer: &mut P, use_ansi_color: bool) {
        for item in self {
            item.print(printer, use_ansi_color);
        }
    }
}
