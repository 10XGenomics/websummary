use std::{io::Read, path::Path};

use crate::components::{GenericTable, TableRow};
use anyhow::Result;
use itertools::Itertools;

impl GenericTable {
    pub fn from_csv_file(path: impl AsRef<Path>, has_headers: bool) -> Result<Self> {
        GenericTable::from_csv_reader(std::fs::read(path)?.as_slice(), has_headers)
    }

    pub fn from_csv_reader(csv_reader: impl Read, has_headers: bool) -> Result<Self> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .from_reader(csv_reader);

        let header = if has_headers {
            Some(
                rdr.headers()?
                    .into_iter()
                    .map(ToString::to_string)
                    .collect(),
            )
        } else {
            None
        };

        let rows = rdr
            .records()
            .map(|record| {
                record.map(|rec| TableRow(rec.into_iter().map(ToString::to_string).collect()))
            })
            .try_collect()?;

        Ok(GenericTable { header, rows })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::components::{GenericTable, TableRow};

    #[test]
    fn test_from_csv_file() -> Result<()> {
        let data = "\
Sample ID,Name,Valid Barcodes
S1,N1,83.2%
S2,N2,89.7%
";
        let svec =
            |v: [&str; 3]| -> Vec<String> { v.into_iter().map(ToString::to_string).collect() };
        assert_eq!(
            GenericTable::from_csv_reader(data.as_bytes(), true)?,
            GenericTable {
                header: Some(svec(["Sample ID", "Name", "Valid Barcodes"])),
                rows: vec![
                    TableRow(svec(["S1", "N1", "83.2%"])),
                    TableRow(svec(["S2", "N2", "89.7%"]))
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn test_from_csv_file2() -> Result<()> {
        let data = "\
S1,N1,83.2%
S2,N2,89.7%
";
        let svec =
            |v: [&str; 3]| -> Vec<String> { v.into_iter().map(ToString::to_string).collect() };
        assert_eq!(
            GenericTable::from_csv_reader(data.as_bytes(), false)?,
            GenericTable {
                header: None,
                rows: vec![
                    TableRow(svec(["S1", "N1", "83.2%"])),
                    TableRow(svec(["S2", "N2", "89.7%"]))
                ]
            }
        );
        Ok(())
    }
}
