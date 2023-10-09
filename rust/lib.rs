use pyo3::prelude::*;

mod view;

#[pymodule]
fn _lib(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    Ok(())
}
