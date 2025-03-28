mod account;
mod transaction;

pub use account::*;
pub use transaction::*;

trait ToCsv {
    fn header_row() -> &'static str;
    fn to_csv_row(&self) -> String;

    fn format_csv_value(s: &str) -> String {
        if s.contains(',') {
            return format!("\"{}\"", s)
        }

        s.to_string()
    }
}

pub trait VecToCsv {
    fn to_csv(&self) -> String;
}

impl<T> VecToCsv for Vec<T>
where
    T: ToCsv,
{
    fn to_csv(&self) -> String {
        let mut csv = T::header_row().to_string();
        for item in self {
            csv.push('\n');
            csv.push_str(&item.to_csv_row());
        }
        csv
    }
}
