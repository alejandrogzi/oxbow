use pyo3::prelude::*;

mod filelike;
mod util;

mod alignment;
mod bed;
mod sequence;
mod variant;
use crate::alignment::{read_bam, read_sam, PyBamScanner, PySamScanner};
use crate::bed::{read_bed, PyBedScanner};
use crate::sequence::{read_fasta, read_fastq, PyFastaScanner, PyFastqScanner};
use crate::variant::{read_bcf, read_vcf, PyBcfScanner, PyVcfScanner};

/////////////////
/// Python module.
/////////////////

#[pymodule]
#[pyo3(name = "oxbow")]
fn oxbow_sandbox(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFastaScanner>()?;
    m.add_class::<PyFastqScanner>()?;
    m.add_class::<PySamScanner>()?;
    m.add_class::<PyBamScanner>()?;
    m.add_class::<PyVcfScanner>()?;
    m.add_class::<PyBcfScanner>()?;
    m.add_class::<PyBedScanner>()?;
    m.add_function(wrap_pyfunction!(read_fasta, m)?)?;
    m.add_function(wrap_pyfunction!(read_fastq, m)?)?;
    m.add_function(wrap_pyfunction!(read_sam, m)?)?;
    m.add_function(wrap_pyfunction!(read_bam, m)?)?;
    m.add_function(wrap_pyfunction!(read_vcf, m)?)?;
    m.add_function(wrap_pyfunction!(read_bcf, m)?)?;
    m.add_function(wrap_pyfunction!(read_bed, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
