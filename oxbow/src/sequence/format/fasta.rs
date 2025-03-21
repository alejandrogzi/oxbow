use std::io::{self, BufRead, Seek};

use arrow::array::RecordBatchReader;
use arrow::datatypes::Schema;
use noodles::core::Region;

use crate::sequence::batch_iterator::{BatchIterator, QueryBatchIterator};
use crate::sequence::model::batch_builder::BatchBuilder;
use crate::sequence::model::field::FASTA_DEFAULT_FIELD_NAMES;

/// A FASTA scanner.
///
/// # Examples
///
/// ```no_run
/// use std::fs::File;
///
/// let inner = File::open("sample.fa")?;
/// let fmt_reader = noodles::fasta::io::Reader::new(inner);
/// let index = noodles::fasta::fai::read("sample.fa.fai")?;
///
/// let scanner = Scanner::new();
/// let regions = vec!["chr1:1-1000", "chr1:1001-2000", "chr1:2001-3000", "chr1:3001-4000"];
/// let regions: Vec<Region> = regions.iter().map(|s| s.parse().unwrap()).collect();
/// let batches = scanner.scan_query(fmt_reader, index, regions, None, Some(2));
/// ```
pub struct Scanner {}

impl Scanner {
    // Creates a FASTA scanner.
    pub fn new() -> Self {
        Self {}
    }

    /// Returns the FASTA field names.
    pub fn field_names(&self) -> Vec<String> {
        FASTA_DEFAULT_FIELD_NAMES
            .iter()
            .map(|&s| s.to_string())
            .collect()
    }

    /// Returns the Arrow schema.
    pub fn schema(&self, fields: Option<Vec<String>>) -> io::Result<Schema> {
        let batch_builder = BatchBuilder::new_fasta(fields, 0)?;
        Ok(batch_builder.get_arrow_schema())
    }
}

impl Scanner {
    /// Returns an iterator yielding record batches.
    ///
    /// The scan will begin at the current position of the reader and will
    /// move the cursor to the end of the last record scanned.
    ///
    /// # Note
    /// Since reference sequences are often large, the default batch size is
    /// set to 1.
    pub fn scan<R: BufRead>(
        &self,
        fmt_reader: noodles::fasta::io::Reader<R>,
        fields: Option<Vec<String>>,
        batch_size: Option<usize>,
        limit: Option<usize>,
    ) -> io::Result<impl RecordBatchReader> {
        let batch_size = batch_size.unwrap_or(1);
        let batch_builder = BatchBuilder::new_fasta(fields, batch_size)?;
        let batch_iter = BatchIterator::new(fmt_reader, batch_builder, batch_size, limit);
        Ok(batch_iter)
    }

    /// Fetches sequence slice records from a FASTA file by genomic range.
    ///
    /// To read from a BGZF-compressed FASTA file, use `R`: [`noodles::bgzf::IndexedReader`].
    pub fn scan_query<R: BufRead + Seek>(
        &self,
        fmt_reader: noodles::fasta::io::Reader<R>,
        index: noodles::fasta::fai::Index,
        regions: Vec<Region>,
        fields: Option<Vec<String>>,
        batch_size: Option<usize>,
    ) -> io::Result<impl RecordBatchReader> {
        let batch_size = batch_size.unwrap_or(1024);
        let batch_builder = BatchBuilder::new_fasta(fields, batch_size)?;
        let batch_iter =
            QueryBatchIterator::new(fmt_reader, index, regions, batch_builder, batch_size);
        Ok(batch_iter)
    }
}
