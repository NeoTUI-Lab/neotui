use pyo3::prelude::*;
use neotui_core::components::{Label, VBox};
use neotui_core::component::Component;

#[pyclass]
pub struct PyLabel {
    pub inner: Label,
}

#[pymethods]
impl PyLabel {
    #[new]
    pub fn new(text: String) -> Self {
        PyLabel {
            inner: Label::new(text),
        }
    }

    pub fn get_text(&self) -> String {
        self.inner.text.clone()
    }
}

#[pyclass(unsendable)]
pub struct PyVBox {
    inner: VBox,
}

#[pymethods]
impl PyVBox {
    #[new]
    pub fn new() -> Self {
        PyVBox {
            inner: VBox::new(),
        }
    }

    pub fn add_label(&mut self, label: PyRef<PyLabel>) {
        self.inner.add_child(Box::new(label.inner.clone()));
    }

    pub fn child_count(&self) -> usize {
        self.inner.children_len()
    }
}

#[pymodule]
fn neotui_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyLabel>()?;
    m.add_class::<PyVBox>()?;
    Ok(())
}
