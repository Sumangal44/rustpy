use super::PyObject;
use crate::objects::bool::PyBool;
use crate::objects::bytes::PyBytes;
use crate::objects::int::PyInt;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::none::PyNone;
use crate::objects::string::PyString;
use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::rc::Rc;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[derive(Debug)]
pub enum PyFileKind {
    File(std::fs::File),
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub struct PyFileInner {
    pub kind: PyFileKind,
    pub path: String,
    pub mode: String,
    pub encoding: String,
    pub closed: bool,
    pub eof: bool,
}

pub struct PyFile {
    pub inner: Rc<RefCell<PyFileInner>>,
}

impl fmt::Debug for PyFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl Clone for PyFile {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl PyFile {
    pub fn from_file(path: String, mode: String, encoding: String, file: std::fs::File) -> Self {
        Self {
            inner: Rc::new(RefCell::new(PyFileInner {
                kind: PyFileKind::File(file),
                path,
                mode,
                encoding,
                closed: false,
                eof: false,
            })),
        }
    }

    pub fn stdin() -> Self {
        Self {
            inner: Rc::new(RefCell::new(PyFileInner {
                kind: PyFileKind::Stdin,
                path: "<stdin>".to_string(),
                mode: "r".to_string(),
                encoding: "utf-8".to_string(),
                closed: false,
                eof: false,
            })),
        }
    }

    pub fn stdout() -> Self {
        Self {
            inner: Rc::new(RefCell::new(PyFileInner {
                kind: PyFileKind::Stdout,
                path: "<stdout>".to_string(),
                mode: "w".to_string(),
                encoding: "utf-8".to_string(),
                closed: false,
                eof: false,
            })),
        }
    }

    pub fn stderr() -> Self {
        Self {
            inner: Rc::new(RefCell::new(PyFileInner {
                kind: PyFileKind::Stderr,
                path: "<stderr>".to_string(),
                mode: "w".to_string(),
                closed: false,
                eof: false,
                encoding: "utf-8".to_string(),
            })),
        }
    }

    pub fn write(&self, data: String) -> Result<Rc<dyn PyObject>, String> {
        write_impl(&self.inner, &data)
    }

    pub fn flush(&self) -> Result<Rc<dyn PyObject>, String> {
        flush_impl(&self.inner)
    }
}

fn check_closed(inner: &RefCell<PyFileInner>) -> Result<(), String> {
    if inner.borrow().closed {
        return Err("ValueError: I/O operation on closed file.".to_string());
    }
    Ok(())
}

fn read_impl(
    inner: &RefCell<PyFileInner>,
    size: Option<usize>,
) -> Result<Rc<dyn PyObject>, String> {
    check_closed(inner)?;
    let mut inner_mut = inner.borrow_mut();
    let is_binary = inner_mut.mode.contains('b');
    let encoding = inner_mut.encoding.clone();
    let is_readable = !inner_mut.mode.contains('w')
        && !inner_mut.mode.contains('a')
        && !inner_mut.mode.contains('x')
        || inner_mut.mode.contains('+');

    if !is_readable {
        return Err("io.UnsupportedOperation: not readable".to_string());
    }

    match &mut inner_mut.kind {
        PyFileKind::File(f) => {
            if is_binary {
                if let Some(n) = size {
                    let mut buf = vec![0u8; n];
                    let nread = f.read(&mut buf).map_err(|e| format!("OSError: {}", e))?;
                    buf.truncate(nread);
                    if nread == 0 {
                        inner_mut.eof = true;
                    }
                    Ok(Rc::new(PyBytes::new(buf)))
                } else {
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)
                        .map_err(|e| format!("OSError: {}", e))?;
                    inner_mut.eof = true;
                    Ok(Rc::new(PyBytes::new(buf)))
                }
            } else {
                if let Some(n) = size {
                    let mut buf = vec![0u8; n];
                    let nread = f.read(&mut buf).map_err(|e| format!("OSError: {}", e))?;
                    buf.truncate(nread);
                    if nread == 0 {
                        inner_mut.eof = true;
                    }
                    let s = crate::encoding::decode(&buf, &encoding)?;
                    Ok(Rc::new(PyString::new(s)))
                } else {
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)
                        .map_err(|e| format!("OSError: {}", e))?;
                    inner_mut.eof = true;
                    let s = crate::encoding::decode(&buf, &encoding)?;
                    Ok(Rc::new(PyString::new(s)))
                }
            }
        }
        PyFileKind::Stdin => {
            let mut stdin = io::stdin();
            if is_binary {
                if let Some(n) = size {
                    let mut buf = vec![0u8; n];
                    let nread = stdin
                        .read(&mut buf)
                        .map_err(|e| format!("OSError: {}", e))?;
                    buf.truncate(nread);
                    if nread == 0 {
                        inner_mut.eof = true;
                    }
                    Ok(Rc::new(PyBytes::new(buf)))
                } else {
                    let mut buf = Vec::new();
                    stdin
                        .read_to_end(&mut buf)
                        .map_err(|e| format!("OSError: {}", e))?;
                    inner_mut.eof = true;
                    Ok(Rc::new(PyBytes::new(buf)))
                }
            } else {
                if let Some(n) = size {
                    let mut buf = vec![0u8; n];
                    let nread = stdin
                        .read(&mut buf)
                        .map_err(|e| format!("OSError: {}", e))?;
                    buf.truncate(nread);
                    if nread == 0 {
                        inner_mut.eof = true;
                    }
                    let s =
                        String::from_utf8(buf).map_err(|e| format!("UnicodeDecodeError: {}", e))?;
                    Ok(Rc::new(PyString::new(s)))
                } else {
                    let mut s = String::new();
                    stdin
                        .read_to_string(&mut s)
                        .map_err(|e| format!("OSError: {}", e))?;
                    inner_mut.eof = true;
                    Ok(Rc::new(PyString::new(s)))
                }
            }
        }
        _ => Err("io.UnsupportedOperation: not readable".to_string()),
    }
}

fn read_byte(inner: &RefCell<PyFileInner>) -> Result<Option<u8>, String> {
    let mut inner_mut = inner.borrow_mut();
    if inner_mut.closed {
        return Err("ValueError: I/O operation on closed file.".to_string());
    }
    match &mut inner_mut.kind {
        PyFileKind::File(f) => {
            let mut byte = [0u8; 1];
            match f.read(&mut byte) {
                Ok(0) => Ok(None),
                Ok(_) => Ok(Some(byte[0])),
                Err(e) => Err(format!("OSError: {}", e)),
            }
        }
        PyFileKind::Stdin => {
            let mut byte = [0u8; 1];
            match io::stdin().read(&mut byte) {
                Ok(0) => Ok(None),
                Ok(_) => Ok(Some(byte[0])),
                Err(e) => Err(format!("OSError: {}", e)),
            }
        }
        _ => Err("io.UnsupportedOperation: not readable".to_string()),
    }
}

fn readline_raw(inner: &RefCell<PyFileInner>, max_size: Option<usize>) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let max = max_size.unwrap_or(usize::MAX);
    loop {
        if buf.len() >= max {
            break;
        }
        match read_byte(inner)? {
            Some(b) => {
                buf.push(b);
                if b == b'\n' {
                    break;
                }
            }
            None => break,
        }
    }
    Ok(buf)
}

fn readline_impl(
    inner: &RefCell<PyFileInner>,
    size: Option<usize>,
) -> Result<Rc<dyn PyObject>, String> {
    check_closed(inner)?;
    let is_binary = inner.borrow().mode.contains('b');
    let encoding = inner.borrow().encoding.clone();
    let raw = readline_raw(inner, size)?;

    if raw.is_empty() {
        inner.borrow_mut().eof = true;
        if is_binary {
            return Ok(Rc::new(PyBytes::new(Vec::new())));
        } else {
            return Ok(Rc::new(PyString::new(String::new())));
        }
    }

    if is_binary {
        Ok(Rc::new(PyBytes::new(raw)))
    } else {
        let s = crate::encoding::decode(&raw, &encoding)?;
        Ok(Rc::new(PyString::new(s)))
    }
}

fn readlines_impl(inner: &RefCell<PyFileInner>) -> Result<Rc<dyn PyObject>, String> {
    check_closed(inner)?;
    let is_binary = inner.borrow().mode.contains('b');
    let mut lines: Vec<Rc<dyn PyObject>> = Vec::new();
    loop {
        let raw = readline_raw(inner, None)?;
        if raw.is_empty() {
            break;
        }
        if is_binary {
            lines.push(Rc::new(PyBytes::new(raw)) as Rc<dyn PyObject>);
        } else {
            let s = String::from_utf8(raw).map_err(|e| format!("UnicodeDecodeError: {}", e))?;
            lines.push(Rc::new(PyString::new(s)) as Rc<dyn PyObject>);
        }
    }
    inner.borrow_mut().eof = true;
    Ok(Rc::new(crate::objects::list::PyList::new(lines)))
}

fn flush_impl(inner: &RefCell<PyFileInner>) -> Result<Rc<dyn PyObject>, String> {
    check_closed(inner)?;
    let mut inner_mut = inner.borrow_mut();
    match &mut inner_mut.kind {
        PyFileKind::File(f) => {
            f.flush().map_err(|e| format!("OSError: {}", e))?;
        }
        PyFileKind::Stdout => {
            io::stdout()
                .flush()
                .map_err(|e| format!("OSError: {}", e))?;
        }
        PyFileKind::Stderr => {
            io::stderr()
                .flush()
                .map_err(|e| format!("OSError: {}", e))?;
        }
        _ => {}
    }
    Ok(Rc::new(PyNone::new()))
}

fn write_impl(inner: &RefCell<PyFileInner>, data: &str) -> Result<Rc<dyn PyObject>, String> {
    check_closed(inner)?;
    let mut inner_mut = inner.borrow_mut();
    let is_writable = inner_mut.mode.contains('w')
        || inner_mut.mode.contains('a')
        || inner_mut.mode.contains('x')
        || inner_mut.mode.contains('+');
    if !is_writable {
        return Err("io.UnsupportedOperation: not writable".to_string());
    }
    let encoding = inner_mut.encoding.clone();
    match &mut inner_mut.kind {
        PyFileKind::File(f) => {
            let bytes = crate::encoding::encode(data, &encoding)?;
            let n = f.write(&bytes).map_err(|e| format!("OSError: {}", e))?;
            Ok(Rc::new(PyInt::from_i64(n as i64)))
        }
        PyFileKind::Stdout => {
            print!("{}", data);
            io::stdout().flush().ok();
            Ok(Rc::new(PyInt::from_i64(data.len() as i64)))
        }
        PyFileKind::Stderr => {
            eprint!("{}", data);
            io::stderr().flush().ok();
            Ok(Rc::new(PyInt::from_i64(data.len() as i64)))
        }
        _ => Err("io.UnsupportedOperation: not writable".to_string()),
    }
}

fn write_bytes_impl(inner: &RefCell<PyFileInner>, data: &[u8]) -> Result<Rc<dyn PyObject>, String> {
    check_closed(inner)?;
    let mut inner_mut = inner.borrow_mut();
    let is_writable = inner_mut.mode.contains('w')
        || inner_mut.mode.contains('a')
        || inner_mut.mode.contains('x')
        || inner_mut.mode.contains('+');
    if !is_writable {
        return Err("io.UnsupportedOperation: not writable".to_string());
    }
    match &mut inner_mut.kind {
        PyFileKind::File(f) => {
            let n = f.write(data).map_err(|e| format!("OSError: {}", e))?;
            Ok(Rc::new(PyInt::from_i64(n as i64)))
        }
        PyFileKind::Stdout => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            let n = handle.write(data).map_err(|e| format!("OSError: {}", e))?;
            Ok(Rc::new(PyInt::from_i64(n as i64)))
        }
        PyFileKind::Stderr => {
            let stderr = io::stderr();
            let mut handle = stderr.lock();
            let n = handle.write(data).map_err(|e| format!("OSError: {}", e))?;
            Ok(Rc::new(PyInt::from_i64(n as i64)))
        }
        _ => Err("io.UnsupportedOperation: not writable".to_string()),
    }
}

fn close_impl(inner: &RefCell<PyFileInner>) -> Result<Rc<dyn PyObject>, String> {
    let mut inner_mut = inner.borrow_mut();
    if inner_mut.closed {
        return Ok(Rc::new(PyNone::new()));
    }
    inner_mut.closed = true;
    if let PyFileKind::File(f) = &mut inner_mut.kind {
        let _ = f.flush();
    }
    Ok(Rc::new(PyNone::new()))
}

use std::io::IsTerminal;

fn isatty_fd(fd: i32) -> bool {
    // Wrap the fd in a std::fs::File for IsTerminal check
    // On Unix, we can use std::os::unix::io::FromRawFd, but that's unsafe
    // Instead, just use the platform APIs
    #[cfg(unix)]
    {
        use std::os::unix::io::FromRawFd;
        // SAFETY: We're just checking if it's a terminal, not doing I/O
        let f = unsafe { std::fs::File::from_raw_fd(fd) };
        let result = f.is_terminal();
        std::mem::forget(f); // Don't close the fd
        result
    }
    #[cfg(not(unix))]
    {
        let _ = fd;
        false
    }
}

impl PyObject for PyFile {
    fn get_type(&self) -> &'static str {
        "file"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        let b = self.inner.borrow();
        let encoding = if b.mode.contains('b') {
            String::new()
        } else {
            " encoding='UTF-8'".to_string()
        };
        format!(
            "<_io.TextIOWrapper name='{}' mode='{}'{}>",
            b.path, b.mode, encoding
        )
    }

    fn is_truthy(&self) -> bool {
        !self.inner.borrow().closed
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        check_closed(&self.inner)?;
        Ok(Rc::new(PyFileLineIterator {
            inner: Rc::clone(&self.inner),
        }))
    }

    fn get_attr(&self, attr: &str) -> Result<Rc<dyn PyObject>, String> {
        let inner = Rc::clone(&self.inner);
        let mode = self.inner.borrow().mode.clone();
        let is_binary = mode.contains('b');

        match attr {
            "read" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "read".to_string(),
                move |args| {
                    let size = if args.len() >= 1 {
                        if let Some(i) = args[0].as_any().downcast_ref::<PyInt>() {
                            let n = i.as_i64().unwrap_or(-1);
                            if n < 0 { None } else { Some(n as usize) }
                        } else {
                            return Err(format!(
                                "TypeError: argument must be int, not '{}'",
                                args[0].get_type()
                            ));
                        }
                    } else {
                        None
                    };
                    read_impl(&inner, size)
                },
            ))),
            "readline" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "readline".to_string(),
                move |args| {
                    let size = if args.len() >= 1 {
                        if let Some(i) = args[0].as_any().downcast_ref::<PyInt>() {
                            Some(i.as_i64().unwrap_or(-1) as usize)
                        } else {
                            return Err(format!(
                                "TypeError: argument must be int, not '{}'",
                                args[0].get_type()
                            ));
                        }
                    } else {
                        None
                    };
                    readline_impl(&inner, size)
                },
            ))),
            "readlines" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "readlines".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: readlines() takes no arguments".to_string());
                    }
                    readlines_impl(&inner)
                },
            ))),
            "write" => {
                if is_binary {
                    Ok(Rc::new(PyNativeFunction::new_pos_only(
                        "write".to_string(),
                        move |args| {
                            if args.is_empty() {
                                return Err(
                                    "TypeError: write() takes at least 1 argument".to_string()
                                );
                            }
                            if let Some(b) = args[0].as_any().downcast_ref::<PyBytes>() {
                                write_bytes_impl(&inner, &b.value)
                            } else if let Some(s) = args[0].as_any().downcast_ref::<PyString>() {
                                write_impl(&inner, &s.value)
                            } else {
                                write_impl(&inner, &args[0].str())
                            }
                        },
                    )))
                } else {
                    Ok(Rc::new(PyNativeFunction::new_pos_only(
                        "write".to_string(),
                        move |args| {
                            if args.is_empty() {
                                return Err(
                                    "TypeError: write() takes at least 1 argument".to_string()
                                );
                            }
                            write_impl(&inner, &args[0].str())
                        },
                    )))
                }
            }
            "writelines" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "writelines".to_string(),
                move |args| {
                    if args.is_empty() {
                        return Err("TypeError: writelines() takes at least 1 argument".to_string());
                    }
                    let iter = args[0].get_iter()?;
                    while let Some(item) = iter.get_next()? {
                        if is_binary {
                            if let Some(b) = item.as_any().downcast_ref::<PyBytes>() {
                                write_bytes_impl(&inner, &b.value)?;
                            } else {
                                write_impl(&inner, &item.str())?;
                            }
                        } else {
                            write_impl(&inner, &item.str())?;
                        }
                    }
                    Ok(Rc::new(PyNone::new()))
                },
            ))),
            "close" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "close".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: close() takes no arguments".to_string());
                    }
                    close_impl(&inner)
                },
            ))),
            "flush" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "flush".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: flush() takes no arguments".to_string());
                    }
                    flush_impl(&inner)
                },
            ))),
            "seek" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "seek".to_string(),
                move |args| {
                    if args.is_empty() || args.len() > 2 {
                        return Err("TypeError: seek() takes 1-2 arguments".to_string());
                    }
                    let offset = if let Some(i) = args[0].as_any().downcast_ref::<PyInt>() {
                        i.as_i64().unwrap_or(0)
                    } else {
                        return Err("TypeError: integer argument expected".to_string());
                    };
                    let whence = if args.len() >= 2 {
                        if let Some(i) = args[1].as_any().downcast_ref::<PyInt>() {
                            i.as_i64().unwrap_or(0)
                        } else {
                            return Err("TypeError: integer argument expected".to_string());
                        }
                    } else {
                        0
                    };
                    let seek_from = match whence {
                        0 => SeekFrom::Start(offset as u64),
                        1 => SeekFrom::Current(offset),
                        2 => SeekFrom::End(offset),
                        _ => return Err("ValueError: invalid whence value".to_string()),
                    };
                    check_closed(&inner)?;
                    let mut inner_mut = inner.borrow_mut();
                    match &mut inner_mut.kind {
                        PyFileKind::File(f) => {
                            let pos = f.seek(seek_from).map_err(|e| format!("OSError: {}", e))?;
                            inner_mut.eof = false;
                            Ok(Rc::new(PyInt::from_i64(pos as i64)))
                        }
                        _ => Err("io.UnsupportedOperation: seek not supported".to_string()),
                    }
                },
            ))),
            "tell" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "tell".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: tell() takes no arguments".to_string());
                    }
                    check_closed(&inner)?;
                    let mut inner_mut = inner.borrow_mut();
                    match &mut inner_mut.kind {
                        PyFileKind::File(f) => {
                            let pos = f
                                .seek(SeekFrom::Current(0))
                                .map_err(|e| format!("OSError: {}", e))?;
                            Ok(Rc::new(PyInt::from_i64(pos as i64)))
                        }
                        _ => Err("io.UnsupportedOperation: tell not supported".to_string()),
                    }
                },
            ))),
            "fileno" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "fileno".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: fileno() takes no arguments".to_string());
                    }
                    check_closed(&inner)?;
                    let inner_mut = inner.borrow();
                    match &inner_mut.kind {
                        PyFileKind::File(f) => {
                            #[cfg(unix)]
                            {
                                Ok(Rc::new(PyInt::from_i64(f.as_raw_fd() as i64)))
                            }
                            #[cfg(not(unix))]
                            {
                                Ok(Rc::new(PyInt::from_i64(0)))
                            }
                        }
                        PyFileKind::Stdin => Ok(Rc::new(PyInt::from_i64(0))),
                        PyFileKind::Stdout => Ok(Rc::new(PyInt::from_i64(1))),
                        PyFileKind::Stderr => Ok(Rc::new(PyInt::from_i64(2))),
                    }
                },
            ))),
            "isatty" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "isatty".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: isatty() takes no arguments".to_string());
                    }
                    check_closed(&inner)?;
                    let inner_mut = inner.borrow();
                    match &inner_mut.kind {
                        PyFileKind::File(f) => {
                            #[cfg(unix)]
                            {
                                Ok(Rc::new(PyBool::new(isatty_fd(f.as_raw_fd()))))
                            }
                            #[cfg(not(unix))]
                            {
                                let _ = f;
                                Ok(Rc::new(PyBool::new(false)))
                            }
                        }
                        PyFileKind::Stdin => Ok(Rc::new(PyBool::new(isatty_fd(0)))),
                        PyFileKind::Stdout => Ok(Rc::new(PyBool::new(isatty_fd(1)))),
                        PyFileKind::Stderr => Ok(Rc::new(PyBool::new(isatty_fd(2)))),
                    }
                },
            ))),
            "readable" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "readable".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: readable() takes no arguments".to_string());
                    }
                    let inner_mut = inner.borrow();
                    let m = &inner_mut.mode;
                    let r =
                        !m.contains('w') && !m.contains('a') && !m.contains('x') || m.contains('+');
                    Ok(Rc::new(PyBool::new(r)))
                },
            ))),
            "writable" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "writable".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: writable() takes no arguments".to_string());
                    }
                    let inner_mut = inner.borrow();
                    let m = &inner_mut.mode;
                    let w =
                        m.contains('w') || m.contains('a') || m.contains('x') || m.contains('+');
                    Ok(Rc::new(PyBool::new(w)))
                },
            ))),
            "seekable" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "seekable".to_string(),
                move |args| {
                    if !args.is_empty() {
                        return Err("TypeError: seekable() takes no arguments".to_string());
                    }
                    let inner_mut = inner.borrow();
                    match &inner_mut.kind {
                        PyFileKind::File(_) => Ok(Rc::new(PyBool::new(true))),
                        _ => Ok(Rc::new(PyBool::new(false))),
                    }
                },
            ))),
            "__enter__" => {
                let inner_clone = Rc::clone(&self.inner);
                Ok(Rc::new(PyNativeFunction::new_pos_only(
                    "__enter__".to_string(),
                    move |args| {
                        if !args.is_empty() {
                            return Err("TypeError: __enter__() takes no arguments".to_string());
                        }
                        check_closed(&inner_clone)?;
                        Ok(Rc::new(PyFile {
                            inner: Rc::clone(&inner_clone),
                        }) as Rc<dyn PyObject>)
                    },
                )))
            }
            "__exit__" => Ok(Rc::new(PyNativeFunction::new_pos_only(
                "__exit__".to_string(),
                move |_args| {
                    close_impl(&inner)?;
                    Ok(Rc::new(PyNone::new()))
                },
            ))),
            "name" => {
                let path = self.inner.borrow().path.clone();
                Ok(Rc::new(PyString::new(path)))
            }
            "mode" => {
                let mode = self.inner.borrow().mode.clone();
                Ok(Rc::new(PyString::new(mode)))
            }
            "closed" => {
                let closed = self.inner.borrow().closed;
                Ok(Rc::new(PyBool::new(closed)))
            }
            _ => {
                // Try to match by creating a native function that returns the attribute
                Err(format!(
                    "AttributeError: '{}' object has no attribute '{}'",
                    self.get_type(),
                    attr
                ))
            }
        }
    }

    fn set_attr(&self, attr: &str, _value: Rc<dyn PyObject>) -> Result<(), String> {
        Err(format!(
            "AttributeError: '{}' object has no attribute '{}' (or is read-only)",
            self.get_type(),
            attr
        ))
    }

    fn hash(&self) -> Result<i64, String> {
        Err(format!("TypeError: unhashable type: '{}'", self.get_type()))
    }
}

#[derive(Clone)]
pub struct PyFileLineIterator {
    pub inner: Rc<RefCell<PyFileInner>>,
}

impl fmt::Debug for PyFileLineIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl PyObject for PyFileLineIterator {
    fn get_type(&self) -> &'static str {
        "file_line_iterator"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr(&self) -> String {
        format!("<file_line_iterator object at {:p}>", self)
    }

    fn get_iter(&self) -> Result<Rc<dyn PyObject>, String> {
        Ok(Rc::new(self.clone()))
    }

    fn get_next(&self) -> Result<Option<Rc<dyn PyObject>>, String> {
        let is_binary = self.inner.borrow().mode.contains('b');
        let raw = readline_raw(&self.inner, None)?;
        if raw.is_empty() {
            return Ok(None);
        }
        if is_binary {
            Ok(Some(Rc::new(PyBytes::new(raw)) as Rc<dyn PyObject>))
        } else {
            let s = String::from_utf8(raw).map_err(|e| format!("UnicodeDecodeError: {}", e))?;
            Ok(Some(Rc::new(PyString::new(s)) as Rc<dyn PyObject>))
        }
    }

    fn hash(&self) -> Result<i64, String> {
        Err(format!("TypeError: unhashable type: '{}'", self.get_type()))
    }
}
