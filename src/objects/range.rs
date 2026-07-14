use super::PyObject;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct PyRange {
    pub start: i64,
    pub stop: i64,
    pub step: i64,
}

#[derive(Clone)]
pub struct PyRangeIterator {
    pub current: RefCell<i64>,
    pub stop: i64,
    pub step: i64,
}

impl PyRange {
    pub fn new(start: i64, stop: i64, step: i64) -> Self {
        Self { start, stop, step }
    }

    pub fn len(&self) -> usize {
        if self.step > 0 && self.start >= self.stop {
            return 0;
        }
        if self.step < 0 && self.start <= self.stop {
            return 0;
        }
        let diff = self.stop - self.start;
        let raw = diff.abs() / self.step.abs();
        let extra = if diff.abs() % self.step.abs() != 0 { 1 } else { 0 };
        (raw + extra) as usize
    }
}

impl std::fmt::Debug for PyRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "range({}, {})", self.start, self.stop)
    }
}

impl PyObject for PyRange {
    fn get_type(&self) -> &'static str {
        "range"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        if self.step == 1 {
            format!("range({}, {})", self.start, self.stop)
        } else {
            format!("range({}, {}, {})", self.start, self.stop, self.step)
        }
    }

    fn is_truthy(&self) -> bool {
        self.len() > 0
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(PyRangeIterator {
            current: RefCell::new(self.start),
            stop: self.stop,
            step: self.step,
        }))
    }

    fn get_item(&self, index: Rc<dyn PyObject>) -> Result<Rc<dyn PyObject>, String> {
        let idx = if let Some(i) = index.as_any().downcast_ref::<super::int::PyInt>() {
            i.as_i64().unwrap_or(0)
        } else {
            return Err("TypeError: range indices must be integers".to_string());
        };
        let len = self.len() as i64;
        let actual = if idx < 0 { idx + len } else { idx };
        if actual < 0 || actual >= len {
            return Err("IndexError: range index out of range".to_string());
        }
        Ok(Rc::new(super::int::PyInt::from_i64(self.start + actual * self.step)))
    }

    fn contains(&self, other: Rc<dyn PyObject>) -> Result<bool, String> {
        let val = if let Some(i) = other.as_any().downcast_ref::<super::int::PyInt>() {
            i.as_i64().unwrap_or(0)
        } else {
            return Err(format!("TypeError: '{}' object cannot be interpreted as an integer", other.get_type()));
        };
        if self.step > 0 {
            Ok(val >= self.start && val < self.stop && (val - self.start) % self.step == 0)
        } else if self.step < 0 {
            Ok(val <= self.start && val > self.stop && (val - self.start) % self.step == 0)
        } else {
            Ok(false)
        }
    }
}

impl std::fmt::Debug for PyRangeIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "range_iterator")
    }
}

impl PyObject for PyRangeIterator {
    fn get_type(&self) -> &'static str {
        "range_iterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        "range_iterator".to_string()
    }

    fn is_truthy(&self) -> bool {
        true
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let mut current = self.current.borrow_mut();
        if (self.step > 0 && *current >= self.stop) || (self.step < 0 && *current <= self.stop) {
            return Ok(None);
        }
        let result = Rc::new(super::int::PyInt::from_i64(*current)) as Rc<dyn PyObject>;
        *current += self.step;
        Ok(Some(result))
    }
}
