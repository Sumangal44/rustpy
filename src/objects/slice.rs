use super::PyObject;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct PySlice {
    pub start: Option<i64>,
    pub stop: Option<i64>,
    pub step: Option<i64>,
}

impl PySlice {
    pub fn new(start: Option<i64>, stop: Option<i64>, step: Option<i64>) -> Self {
        Self { start, stop, step }
    }

    pub fn resolve(&self, length: usize) -> (usize, usize, i64) {
        let step = self.step.unwrap_or(1);
        let start = match self.start {
            Some(s) => {
                if s < 0 {
                    let adjusted = length as i64 + s;
                    if adjusted < 0 { 0 } else { adjusted as usize }
                } else {
                    s as usize
                }
            }
            None => {
                if step > 0 { 0 } else { length - 1 }
            }
        };
        let stop = match self.stop {
            Some(s) => {
                if s < 0 {
                    let adjusted = length as i64 + s;
                    if adjusted < 0 { 0 } else { adjusted as usize }
                } else {
                    s as usize
                }
            }
            None => {
                if step > 0 { length } else { 0 }
            }
        };
        (start.min(length), stop.min(length).max(1), step)
    }
}

impl PyObject for PySlice {
    fn get_type(&self) -> &'static str {
        "slice"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let s = self.start.map(|v| v.to_string()).unwrap_or_else(|| "None".to_string());
        let e = self.stop.map(|v| v.to_string()).unwrap_or_else(|| "None".to_string());
        let p = self.step.map(|v| v.to_string()).unwrap_or_else(|| "None".to_string());
        format!("slice({}, {}, {})", s, e, p)
    }

    fn hash(&self) -> Result<i64, String> {
        Err("TypeError: unhashable type: 'slice'".to_string())
    }
}
