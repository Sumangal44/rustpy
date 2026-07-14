use crate::objects::PyObject;
use crate::objects::bool::PyBool;
use crate::objects::float::PyFloat;
use crate::objects::int::PyInt;
use crate::objects::module::PyModule;
use crate::objects::native_function::PyNativeFunction;
use crate::objects::tuple::PyTuple;
use std::rc::Rc;

fn to_f64(obj: &Rc<dyn PyObject>) -> Result<f64, String> {
    if let Some(i) = obj.as_any().downcast_ref::<PyInt>() {
        i.as_i64().map(|v| v as f64).ok_or_else(|| "OverflowError: int too large to convert to float".to_string())
    } else if let Some(f) = obj.as_any().downcast_ref::<PyFloat>() {
        Ok(f.value)
    } else {
        Err(format!("TypeError: must be real number, not '{}'", obj.get_type()))
    }
}

fn to_i64(obj: &Rc<dyn PyObject>) -> Result<i64, String> {
    if let Some(i) = obj.as_any().downcast_ref::<PyInt>() {
        i.as_i64().ok_or_else(|| "OverflowError: int too large".to_string())
    } else {
        Err(format!("TypeError: must be int, not '{}'", obj.get_type()))
    }
}

fn gcd_i64(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd_i64(b, a % b) }
}

pub fn create_math_module() -> Rc<PyModule> {
    let module = PyModule::new("math".to_string());
    let module = Rc::new(module);

    module.set_attr_inner("pi", Rc::new(PyFloat::new(std::f64::consts::PI)) as Rc<dyn PyObject>);
    module.set_attr_inner("e", Rc::new(PyFloat::new(std::f64::consts::E)) as Rc<dyn PyObject>);
    module.set_attr_inner("tau", Rc::new(PyFloat::new(std::f64::consts::TAU)) as Rc<dyn PyObject>);
    module.set_attr_inner("inf", Rc::new(PyFloat::new(f64::INFINITY)) as Rc<dyn PyObject>);
    module.set_attr_inner("nan", Rc::new(PyFloat::new(f64::NAN)) as Rc<dyn PyObject>);

    let math_e = std::f64::consts::E;

    macro_rules! math_fn1 {
        ($name:expr, $func:expr) => {
            module.set_attr_inner($name, Rc::new(PyNativeFunction::new_pos_only($name.to_string(), move |args| {
                if args.len() != 1 {
                    return Err(format!("TypeError: {}() takes exactly one argument ({} given)", $name, args.len()));
                }
                let x = to_f64(&args[0])?;
                Ok(Rc::new(PyFloat::new($func(x))))
            })) as Rc<dyn PyObject>)
        };
    }

    macro_rules! math_fn2 {
        ($name:expr, $func:expr) => {
            module.set_attr_inner($name, Rc::new(PyNativeFunction::new_pos_only($name.to_string(), move |args| {
                if args.len() != 2 {
                    return Err(format!("TypeError: {}() takes exactly 2 arguments ({} given)", $name, args.len()));
                }
                let x = to_f64(&args[0])?;
                let y = to_f64(&args[1])?;
                Ok(Rc::new(PyFloat::new($func(x, y))))
            })) as Rc<dyn PyObject>)
        };
    }

    math_fn1!("sqrt", f64::sqrt);
    math_fn1!("sin", f64::sin);
    math_fn1!("cos", f64::cos);
    math_fn1!("tan", f64::tan);
    math_fn1!("asin", f64::asin);
    math_fn1!("acos", f64::acos);
    math_fn1!("atan", f64::atan);
    math_fn2!("atan2", f64::atan2);
    math_fn1!("sinh", f64::sinh);
    math_fn1!("cosh", f64::cosh);
    math_fn1!("tanh", f64::tanh);
    math_fn1!("ceil", f64::ceil);
    math_fn1!("floor", f64::floor);
    math_fn1!("exp", f64::exp);
    math_fn1!("expm1", f64::exp_m1);
    math_fn1!("log10", f64::log10);
    math_fn1!("log1p", f64::ln_1p);
    math_fn1!("log2", f64::log2);
    math_fn2!("pow", f64::powf);
    math_fn2!("hypot", f64::hypot);
    math_fn2!("copysign", f64::copysign);
    // remainder(x, y) = x - y * round(x / y)
    module.set_attr_inner("remainder", Rc::new(PyNativeFunction::new_pos_only("remainder".to_string(), |args| {
        if args.len() != 2 {
            return Err(format!("TypeError: remainder() takes exactly 2 arguments ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        let y = to_f64(&args[1])?;
        if y == 0.0 {
            return Err("ValueError: remainder() argument y must not be zero".to_string());
        }
        Ok(Rc::new(PyFloat::new(x - y * (x / y).round())))
    })) as Rc<dyn PyObject>);
    math_fn1!("erf", erf);
    math_fn1!("erfc", erfc);
    math_fn1!("gamma", gamma);
    math_fn1!("lgamma", lgamma);
    math_fn2!("ldexp", |x, exp| x * (2.0f64).powf(exp));

    // trunc(x) -> return as float (int in Python, but we return float since input is float)
    module.set_attr_inner("trunc", Rc::new(PyNativeFunction::new_pos_only("trunc".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: trunc() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        Ok(Rc::new(PyFloat::new(x.trunc())))
    })) as Rc<dyn PyObject>);

    // fabs(x)
    module.set_attr_inner("fabs", Rc::new(PyNativeFunction::new_pos_only("fabs".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: fabs() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        Ok(Rc::new(PyFloat::new(x.abs())))
    })) as Rc<dyn PyObject>);

    // degrees(x)
    module.set_attr_inner("degrees", Rc::new(PyNativeFunction::new_pos_only("degrees".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: degrees() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        Ok(Rc::new(PyFloat::new(x * 180.0 / std::f64::consts::PI)))
    })) as Rc<dyn PyObject>);

    // radians(x)
    module.set_attr_inner("radians", Rc::new(PyNativeFunction::new_pos_only("radians".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: radians() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        Ok(Rc::new(PyFloat::new(x * std::f64::consts::PI / 180.0)))
    })) as Rc<dyn PyObject>);

    // isclose(a, b, rel_tol=1e-09, abs_tol=0.0)
    module.set_attr_inner("isclose", Rc::new(PyNativeFunction::new("isclose".to_string(), |args, kwargs| {
        if args.len() < 2 || args.len() > 4 {
            return Err(format!("TypeError: isclose() takes 2-4 arguments ({} given)", args.len()));
        }
        let a = to_f64(&args[0])?;
        let b = to_f64(&args[1])?;
        let rel_tol = if args.len() >= 3 { to_f64(&args[2])? } else { kwargs.get("rel_tol").map_or(1e-9, |v| to_f64(v).unwrap_or(1e-9)) };
        let abs_tol = if args.len() >= 4 { to_f64(&args[3])? } else { kwargs.get("abs_tol").map_or(0.0, |v| to_f64(v).unwrap_or(0.0)) };
        let diff = (a - b).abs();
        let result = diff <= (rel_tol * b.abs().max(a.abs())).max(abs_tol);
        Ok(Rc::new(PyBool::new(result)))
    })) as Rc<dyn PyObject>);

    // isfinite(x)
    module.set_attr_inner("isfinite", Rc::new(PyNativeFunction::new_pos_only("isfinite".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: isfinite() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        Ok(Rc::new(PyBool::new(x.is_finite())))
    })) as Rc<dyn PyObject>);

    // isinf(x)
    module.set_attr_inner("isinf", Rc::new(PyNativeFunction::new_pos_only("isinf".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: isinf() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        Ok(Rc::new(PyBool::new(x.is_infinite())))
    })) as Rc<dyn PyObject>);

    // isnan(x)
    module.set_attr_inner("isnan", Rc::new(PyNativeFunction::new_pos_only("isnan".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: isnan() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        Ok(Rc::new(PyBool::new(x.is_nan())))
    })) as Rc<dyn PyObject>);

    // fmod(x, y) -> x % y (IEEE remainder-like, use x % y)
    module.set_attr_inner("fmod", Rc::new(PyNativeFunction::new_pos_only("fmod".to_string(), |args| {
        if args.len() != 2 {
            return Err(format!("TypeError: fmod() takes exactly 2 arguments ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        let y = to_f64(&args[1])?;
        if y == 0.0 {
            return Err("ValueError: fmod() argument y must not be zero".to_string());
        }
        // Python's math.fmod uses C fmod, which is x % y with truncation toward zero
        Ok(Rc::new(PyFloat::new(x % y)))
    })) as Rc<dyn PyObject>);

    // gcd(a, b)
    module.set_attr_inner("gcd", Rc::new(PyNativeFunction::new_pos_only("gcd".to_string(), |args| {
        if args.len() != 2 {
            return Err(format!("TypeError: gcd() takes exactly 2 arguments ({} given)", args.len()));
        }
        let a = to_i64(&args[0])?;
        let b = to_i64(&args[1])?;
        Ok(Rc::new(PyInt::from_i64(gcd_i64(a, b))))
    })) as Rc<dyn PyObject>);

    // lcm(a, b)
    module.set_attr_inner("lcm", Rc::new(PyNativeFunction::new_pos_only("lcm".to_string(), |args| {
        if args.len() != 2 {
            return Err(format!("TypeError: lcm() takes exactly 2 arguments ({} given)", args.len()));
        }
        let a = to_i64(&args[0])?;
        let b = to_i64(&args[1])?;
        if a == 0 || b == 0 {
            return Ok(Rc::new(PyInt::from_i64(0)));
        }
        Ok(Rc::new(PyInt::from_i64(a.abs() / gcd_i64(a, b) * b.abs())))
    })) as Rc<dyn PyObject>);

    // factorial(x)
    module.set_attr_inner("factorial", Rc::new(PyNativeFunction::new_pos_only("factorial".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: factorial() takes exactly one argument ({} given)", args.len()));
        }
        let n = to_i64(&args[0])?;
        if n < 0 {
            return Err("ValueError: factorial() not defined for negative values".to_string());
        }
        let mut result: i64 = 1;
        for i in 2..=n {
            result = result.checked_mul(i).ok_or_else(|| "OverflowError: factorial too large".to_string())?;
        }
        Ok(Rc::new(PyInt::from_i64(result)))
    })) as Rc<dyn PyObject>);

    // comb(n, k)
    module.set_attr_inner("comb", Rc::new(PyNativeFunction::new_pos_only("comb".to_string(), |args| {
        if args.len() != 2 {
            return Err(format!("TypeError: comb() takes exactly 2 arguments ({} given)", args.len()));
        }
        let n = to_i64(&args[0])?;
        let k = to_i64(&args[1])?;
        if n < 0 || k < 0 {
            return Err("ValueError: comb() not defined for negative values".to_string());
        }
        if k > n {
            return Ok(Rc::new(PyInt::from_i64(0)));
        }
        let k = k.min(n - k);
        let mut result: i64 = 1;
        for i in 1..=k {
            result = result.checked_mul(n - k + i).ok_or_else(|| "OverflowError: comb too large".to_string())?;
            result /= i;
        }
        Ok(Rc::new(PyInt::from_i64(result)))
    })) as Rc<dyn PyObject>);

    // perm(n, k)
    module.set_attr_inner("perm", Rc::new(PyNativeFunction::new_pos_only("perm".to_string(), |args| {
        if args.len() != 2 {
            return Err(format!("TypeError: perm() takes exactly 2 arguments ({} given)", args.len()));
        }
        let n = to_i64(&args[0])?;
        let k = to_i64(&args[1])?;
        if n < 0 || k < 0 {
            return Err("ValueError: perm() not defined for negative values".to_string());
        }
        if k > n {
            return Ok(Rc::new(PyInt::from_i64(0)));
        }
        let mut result: i64 = 1;
        for i in (n - k + 1)..=n {
            result = result.checked_mul(i).ok_or_else(|| "OverflowError: perm too large".to_string())?;
        }
        Ok(Rc::new(PyInt::from_i64(result)))
    })) as Rc<dyn PyObject>);

    // dist(p, q)
    module.set_attr_inner("dist", Rc::new(PyNativeFunction::new_pos_only("dist".to_string(), |args| {
        if args.len() != 2 {
            return Err(format!("TypeError: dist() takes exactly 2 arguments ({} given)", args.len()));
        }
        fn to_coords(obj: &Rc<dyn PyObject>) -> Result<Vec<f64>, String> {
            let iter = obj.get_iter()?;
            let mut coords = Vec::new();
            while let Some(item) = iter.get_next()? {
                coords.push(to_f64(&item)?);
            }
            Ok(coords)
        }
        let p = to_coords(&args[0])?;
        let q = to_coords(&args[1])?;
        if p.len() != q.len() {
            return Err("ValueError: both points must have the same number of dimensions".to_string());
        }
        let sum: f64 = p.iter().zip(q.iter()).map(|(a, b)| (a - b).powi(2)).sum();
        Ok(Rc::new(PyFloat::new(sum.sqrt())))
    })) as Rc<dyn PyObject>);

    // log(x, base=math.e)
    module.set_attr_inner("log", Rc::new(PyNativeFunction::new("log".to_string(), move |args, kwargs| {
        let args = args;
        if args.is_empty() {
            return Err("TypeError: log() takes at least 1 argument (0 given)".to_string());
        }
        let x = to_f64(&args[0])?;
        let base = if args.len() >= 2 {
            to_f64(&args[1])?
        } else if let Some(v) = kwargs.get("base") {
            to_f64(v)?
        } else {
            math_e
        };
        let result = if base == math_e {
            x.ln()
        } else {
            x.log(base)
        };
        Ok(Rc::new(PyFloat::new(result)))
    })) as Rc<dyn PyObject>);

    // frexp(x) -> (mantissa, exponent)
    module.set_attr_inner("frexp", Rc::new(PyNativeFunction::new_pos_only("frexp".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: frexp() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        let (mantissa, exponent) = frexp(x);
        Ok(Rc::new(PyTuple::new(vec![
            Rc::new(PyFloat::new(mantissa)) as Rc<dyn PyObject>,
            Rc::new(PyInt::from_i64(exponent as i64)) as Rc<dyn PyObject>,
        ])))
    })) as Rc<dyn PyObject>);

    // fsum(iterable)
    module.set_attr_inner("fsum", Rc::new(PyNativeFunction::new_pos_only("fsum".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: fsum() takes exactly one argument ({} given)", args.len()));
        }
        let iter = args[0].get_iter()?;
        let mut total: f64 = 0.0;
        while let Some(item) = iter.get_next()? {
            total += to_f64(&item)?;
        }
        Ok(Rc::new(PyFloat::new(total)))
    })) as Rc<dyn PyObject>);

    // prod(iterable, start=1)
    module.set_attr_inner("prod", Rc::new(PyNativeFunction::new("prod".to_string(), |args, kwargs| {
        if args.is_empty() || args.len() > 2 {
            return Err(format!("TypeError: prod() takes 1-2 arguments ({} given)", args.len()));
        }
        let iter = args[0].get_iter()?;
        let start = if args.len() >= 2 { to_f64(&args[1])? } else { kwargs.get("start").map_or(1.0, |v| to_f64(v).unwrap_or(1.0)) };
        let mut total: f64 = start;
        while let Some(item) = iter.get_next()? {
            total *= to_f64(&item)?;
        }
        Ok(Rc::new(PyFloat::new(total)))
    })) as Rc<dyn PyObject>);

    // ulp(x)
    module.set_attr_inner("ulp", Rc::new(PyNativeFunction::new_pos_only("ulp".to_string(), |args| {
        if args.len() != 1 {
            return Err(format!("TypeError: ulp() takes exactly one argument ({} given)", args.len()));
        }
        let x = to_f64(&args[0])?;
        Ok(Rc::new(PyFloat::new(ulp(x))))
    })) as Rc<dyn PyObject>);

    module
}

fn erf(x: f64) -> f64 {
    // Approximation of the error function
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

fn gamma(x: f64) -> f64 {
    // Stirling's approximation / Lanczos approximation
    let g = 7.0;
    let p = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    if x < 0.5 {
        std::f64::consts::PI / (std::f64::consts::PI * x).sin() * gamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut y = p[0];
        for i in 1..p.len() {
            y += p[i] / (x + i as f64);
        }
        let t = x + g + 0.5;
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * y
    }
}

fn lgamma(x: f64) -> f64 {
    gamma(x).abs().ln()
}

fn frexp(x: f64) -> (f64, i32) {
    if x == 0.0 {
        return (0.0, 0);
    }
    if x.is_infinite() || x.is_nan() {
        return (x, 0);
    }
    let bits = x.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    let mantissa_bits = (bits & 0x000fffffffffffff) | 0x3fe0000000000000;
    let mantissa = f64::from_bits(mantissa_bits);
    (mantissa, exponent)
}

fn ulp(x: f64) -> f64 {
    if x.is_nan() || x.is_infinite() {
        return x;
    }
    if x == 0.0 {
        return f64::MIN_POSITIVE;
    }
    let x = x.abs();
    let bits = x.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    if exponent <= -1023 {
        return f64::MIN_POSITIVE;
    }
    let ulp_bits = (exponent + 1023) as u64;
    if ulp_bits >= 0x7ff {
        return x; // overflow, return x itself
    }
    f64::from_bits(ulp_bits << 52)
}
