// Copyright (c) 2017-present PyO3 Project and Contributors
//! Python Iterator Interface.
//! Trait and support implementation for implementing iterators

use crate::callback::IntoPyCallbackOutput;
use crate::derive_utils::TryFromPyCell;
use crate::err::PyResult;
use crate::{ffi, IntoPy, IntoPyPointer, PyClass, PyObject, Python};

/// Python Iterator Interface.
///
/// Check [CPython doc](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_iter)
/// for more.
///
/// # Example
/// The following example shows how to implement a simple Python iterator in Rust which yields
/// the integers 1 to 5, before raising `StopIteration("Ended")`.
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::PyIterProtocol;
/// use pyo3::class::iter::IterNextOutput;
///
/// #[pyclass]
/// struct Iter {
///     count: usize
/// }
///
/// #[pyproto]
/// impl PyIterProtocol for Iter {
///     fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<usize, &'static str> {
///         if slf.count < 5 {
///             slf.count += 1;
///             IterNextOutput::Yield(slf.count)
///         } else {
///             IterNextOutput::Return("Ended")
///         }
///     }
/// }
///
/// # let gil = Python::acquire_gil();
/// # let py = gil.python();
/// # let inst = Py::new(py, Iter { count: 0 }).unwrap();
/// # pyo3::py_run!(py, inst, "assert next(inst) == 1");
/// # // test of StopIteration is done in examples/rustapi_module/pyclass_iter.rs
/// ```
#[allow(unused_variables)]
pub trait PyIterProtocol<'p>: PyClass {
    fn __iter__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyIterIterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __next__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyIterNextProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyIterIterProtocol<'p>: PyIterProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyIterNextProtocol<'p>: PyIterProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyIterNextOutput>;
}

/// Extension trait for proc-macro backend.
#[doc(hidden)]
pub trait PyIterSlots {
    fn get_iter() -> ffi::PyType_Slot
    where
        Self: for<'p> PyIterIterProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_iter,
            pfunc: py_unarys_func!(PyIterIterProtocol, Self::__iter__) as _,
        }
    }
    fn get_iternext() -> ffi::PyType_Slot
    where
        Self: for<'p> PyIterNextProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_tp_iternext,
            pfunc: py_unarys_func!(PyIterNextProtocol, Self::__next__) as _,
        }
    }
}

impl<'p, T> PyIterSlots for T where T: PyIterProtocol<'p> {}

/// Output of `__next__` which can either `yield` the next value in the iteration, or
/// `return` a value to raise `StopIteration` in Python.
///
/// See [`PyIterProtocol`](trait.PyIterProtocol.html) for an example.
pub enum IterNextOutput<T, U> {
    Yield(T),
    Return(U),
}

pub type PyIterNextOutput = IterNextOutput<PyObject, PyObject>;

impl IntoPyCallbackOutput<*mut ffi::PyObject> for PyIterNextOutput {
    fn convert(self, _py: Python) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterNextOutput::Yield(o) => Ok(o.into_ptr()),
            IterNextOutput::Return(opt) => Err(crate::exceptions::PyStopIteration::new_err((opt,))),
        }
    }
}

impl<T, U> IntoPyCallbackOutput<PyIterNextOutput> for IterNextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<PyIterNextOutput> {
        match self {
            IterNextOutput::Yield(o) => Ok(IterNextOutput::Yield(o.into_py(py))),
            IterNextOutput::Return(o) => Ok(IterNextOutput::Return(o.into_py(py))),
        }
    }
}

impl<T> IntoPyCallbackOutput<PyIterNextOutput> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<PyIterNextOutput> {
        match self {
            Some(o) => Ok(PyIterNextOutput::Yield(o.into_py(py))),
            None => Ok(PyIterNextOutput::Return(py.None())),
        }
    }
}
