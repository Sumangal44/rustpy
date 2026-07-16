use crate::objects::PyObject;
use crate::objects::int::PyInt;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::string::PyString;
use std::any::Any;
use std::rc::Rc;

/// A `memoryview` object wraps a byte buffer (bytes or bytearray)
/// and provides efficient access without copying.
#[derive(Clone)]
pub struct PyMemoryView {
    pub buf: Rc<Vec<u8>>,
    pub readonly: bool,
    pub itemsize: usize,
    pub format: String,
    pub offset: usize,
    pub length: usize,
}

impl PyMemoryView {
    pub fn from_bytes(data: Vec<u8>, readonly: bool) -> Self {
        let length = data.len();
        Self {
            buf: Rc::new(data),
            readonly,
            itemsize: 1,
            format: "B".to_string(),
            offset: 0,
            length,
        }
    }
}

impl std::fmt::Debug for PyMemoryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<memory at {:p}>", self)
    }
}

impl PyObject for PyMemoryView {
    fn get_type(&self) -> &'static str {
        "memoryview"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<memory at {:p}>", self)
    }

    fn str(&self) -> String {
        self.repr()
    }

    fn get_item(&self, key: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        if let Some(idx) = key.as_any().downcast_ref::<PyInt>() {
            let i = idx.as_i64().unwrap_or(0);
            let actual = if i < 0 {
                (self.length as i64 + i) as usize
            } else {
                i as usize
            };
            if actual >= self.length {
                return Err(format!(
                    "IndexError: index {} out of bounds for memoryview of length {}",
                    i, self.length
                ));
            }
            let byte_val = self.buf[self.offset + actual];
            Ok(Rc::new(PyInt::from_i64(byte_val as i64)) as Rc<dyn PyObject>)
        } else {
            Err("TypeError: memoryview indices must be integers".to_string())
        }
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let buf = Rc::clone(&self.buf);
        let offset = self.offset;
        let length = self.length;
        let format = self.format.clone();
        let itemsize = self.itemsize;
        let readonly = self.readonly;

        match attr {
            "nbytes" => {
                Ok(Rc::new(PyInt::from_i64((length * itemsize) as i64)) as Rc<dyn PyObject>)
            }
            "itemsize" => Ok(Rc::new(PyInt::from_i64(itemsize as i64)) as Rc<dyn PyObject>),
            "ndim" => Ok(Rc::new(PyInt::from_i64(1)) as Rc<dyn PyObject>),
            "readonly" => {
                Ok(Rc::new(crate::objects::bool::PyBool::new(readonly)) as Rc<dyn PyObject>)
            }
            "format" => Ok(Rc::new(PyString::new(format)) as Rc<dyn PyObject>),
            "shape" => {
                let shape_list =
                    Rc::new(crate::objects::tuple::PyTuple::new(vec![
                        Rc::new(PyInt::from_i64(length as i64)) as Rc<dyn PyObject>,
                    ]));
                Ok(shape_list as Rc<dyn PyObject>)
            }
            "tobytes" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "tobytes".to_string(),
                move |_args| {
                    let slice = &buf[offset..offset + length];
                    Ok(Rc::new(crate::objects::bytes::PyBytes::new(slice.to_vec()))
                        as Rc<dyn PyObject>)
                },
            )) as Rc<dyn PyObject>),
            "tolist" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "tolist".to_string(),
                move |_args| {
                    let items: Vec<Rc<dyn PyObject>> = buf[offset..offset + length]
                        .iter()
                        .map(|&b| Rc::new(PyInt::from_i64(b as i64)) as Rc<dyn PyObject>)
                        .collect();
                    Ok(Rc::new(crate::objects::list::PyList::new(items)) as Rc<dyn PyObject>)
                },
            )) as Rc<dyn PyObject>),
            "hex" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "hex".to_string(),
                move |_args| {
                    let hex: String = buf[offset..offset + length]
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect();
                    Ok(Rc::new(PyString::new(hex)) as Rc<dyn PyObject>)
                },
            )) as Rc<dyn PyObject>),
            "cast" => {
                // cast(fmt) – simplified: only 'B' supported, returns self
                let buf2 = Rc::clone(&self.buf);
                let length2 = self.length;
                let offset2 = self.offset;
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "cast".to_string(),
                    move |_args| {
                        let slice = buf2[offset2..offset2 + length2].to_vec();
                        Ok(Rc::new(PyMemoryView::from_bytes(slice, false)) as Rc<dyn PyObject>)
                    },
                )) as Rc<dyn PyObject>)
            }
            _ => Err(format!(
                "AttributeError: 'memoryview' object has no attribute '{}'",
                attr
            )),
        }
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        let items: Vec<Rc<dyn PyObject>> = self.buf[self.offset..self.offset + self.length]
            .iter()
            .map(|&b| Rc::new(PyInt::from_i64(b as i64)) as Rc<dyn PyObject>)
            .collect();
        let list = Rc::new(crate::objects::list::PyList::new(items));
        list.get_iter()
    }
}
