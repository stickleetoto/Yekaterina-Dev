use serde_json::{Value, json};

const MAX_DIM: usize = 128;
const MAX_ELEMS: usize = 20_000;

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "mat.shape" | "mat.add" | "mat.sub" | "mat.scale" | "mat.mul" |
        "mat.transpose" | "mat.trace" | "mat.det" | "mat.identity" |
        "mat.diagonal" | "mat.row_sum" | "mat.col_sum" | "mat.frobenius" |
        "mat.vecmul" | "mat.outer" | "mat.inverse"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "mat.shape" => {
            if args.len() != 1 { return Err("ARG"); }
            let m = matrix(&args[0])?;
            Ok(json!([m.len(), m[0].len()]))
        }
        "mat.add" => binary_matrix(args, |a, b| a + b),
        "mat.sub" => binary_matrix(args, |a, b| a - b),
        "mat.scale" => scale(args),
        "mat.mul" => multiply(args),
        "mat.transpose" => {
            if args.len() != 1 { return Err("ARG"); }
            Ok(json!(transpose(&matrix(&args[0])?)))
        }
        "mat.trace" => {
            let m = one_matrix(args)?;
            if m.len() != m[0].len() { return Err("SHAPE"); }
            finite((0..m.len()).map(|i| m[i][i]).sum())
        }
        "mat.det" => determinant_op(args),
        "mat.identity" => identity_op(args),
        "mat.diagonal" => diagonal_op(args),
        "mat.row_sum" => {
            let m = one_matrix(args)?;
            let out: Vec<f64> = m.iter().map(|row| row.iter().sum()).collect();
            finite_vec(out)
        }
        "mat.col_sum" => {
            let m = one_matrix(args)?;
            let mut out = vec![0.0; m[0].len()];
            for row in &m { for (j, x) in row.iter().enumerate() { out[j] += x; } }
            finite_vec(out)
        }
        "mat.frobenius" => {
            let m = one_matrix(args)?;
            finite(m.iter().flat_map(|r| r.iter()).map(|x| x * x).sum::<f64>().sqrt())
        }
        "mat.vecmul" => vecmul(args),
        "mat.outer" => outer(args),
        "mat.inverse" => inverse_op(args),
        _ => Err("OP"),
    }
}

fn num(v: &Value) -> Result<f64, &'static str> { v.as_f64().ok_or("TYPE") }

fn vector(v: &Value) -> Result<Vec<f64>, &'static str> {
    let a = v.as_array().ok_or("TYPE")?;
    if a.len() > MAX_ELEMS { return Err("LIMIT"); }
    a.iter().map(num).collect()
}

fn matrix(v: &Value) -> Result<Vec<Vec<f64>>, &'static str> {
    let rows = v.as_array().ok_or("TYPE")?;
    if rows.is_empty() || rows.len() > MAX_DIM { return Err("SHAPE"); }
    let mut out = Vec::with_capacity(rows.len());
    let mut cols = None;
    let mut elems = 0usize;
    for row in rows {
        let xs = row.as_array().ok_or("TYPE")?;
        if xs.is_empty() || xs.len() > MAX_DIM { return Err("SHAPE"); }
        match cols { Some(c) if c != xs.len() => return Err("SHAPE"), None => cols = Some(xs.len()), _ => {} }
        elems = elems.saturating_add(xs.len());
        if elems > MAX_ELEMS { return Err("LIMIT"); }
        out.push(xs.iter().map(num).collect::<Result<Vec<_>, _>>()?);
    }
    Ok(out)
}

fn one_matrix(args: &[Value]) -> Result<Vec<Vec<f64>>, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    matrix(&args[0])
}

fn binary_matrix<F>(args: &[Value], f: F) -> Result<Value, &'static str>
where F: Fn(f64, f64) -> f64 {
    if args.len() != 2 { return Err("ARG"); }
    let a = matrix(&args[0])?;
    let b = matrix(&args[1])?;
    if a.len() != b.len() || a[0].len() != b[0].len() { return Err("SHAPE"); }
    let out: Vec<Vec<f64>> = a.into_iter().zip(b).map(|(ra, rb)| {
        ra.into_iter().zip(rb).map(|(x, y)| f(x, y)).collect()
    }).collect();
    finite_matrix(out)
}

fn scale(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let m = matrix(&args[0])?;
    let s = num(&args[1])?;
    finite_matrix(m.into_iter().map(|r| r.into_iter().map(|x| x * s).collect()).collect())
}

fn multiply(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let a = matrix(&args[0])?;
    let b = matrix(&args[1])?;
    let n = a.len(); let k = a[0].len(); let k2 = b.len(); let m = b[0].len();
    if k != k2 { return Err("SHAPE"); }
    if n.saturating_mul(k).saturating_mul(m) > 2_000_000 { return Err("LIMIT"); }
    let bt = transpose(&b);
    let mut out = vec![vec![0.0; m]; n];
    for i in 0..n {
        for j in 0..m { out[i][j] = a[i].iter().zip(bt[j].iter()).map(|(x, y)| x * y).sum(); }
    }
    finite_matrix(out)
}

fn transpose(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let mut out = vec![vec![0.0; m.len()]; m[0].len()];
    for (i, row) in m.iter().enumerate() { for (j, x) in row.iter().enumerate() { out[j][i] = *x; } }
    out
}

fn determinant_op(args: &[Value]) -> Result<Value, &'static str> {
    let m = one_matrix(args)?;
    if m.len() != m[0].len() || m.len() > 32 { return Err("SHAPE"); }
    finite(determinant(m))
}

fn determinant(mut a: Vec<Vec<f64>>) -> f64 {
    let n = a.len();
    let mut det = 1.0;
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n { if a[r][i].abs() > a[pivot][i].abs() { pivot = r; } }
        if a[pivot][i].abs() <= f64::EPSILON { return 0.0; }
        if pivot != i { a.swap(i, pivot); det = -det; }
        let p = a[i][i]; det *= p;
        for r in (i + 1)..n {
            let factor = a[r][i] / p;
            for c in (i + 1)..n { a[r][c] -= factor * a[i][c]; }
        }
    }
    det
}

fn identity_op(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    let n = args[0].as_u64().ok_or("TYPE")? as usize;
    if n == 0 || n > MAX_DIM || n.saturating_mul(n) > MAX_ELEMS { return Err("LIMIT"); }
    let mut m = vec![vec![0.0; n]; n];
    for (i, row) in m.iter_mut().enumerate() { row[i] = 1.0; }
    Ok(json!(m))
}

fn diagonal_op(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    let v = vector(&args[0])?;
    if v.is_empty() || v.len() > MAX_DIM || v.len().saturating_mul(v.len()) > MAX_ELEMS { return Err("LIMIT"); }
    let mut m = vec![vec![0.0; v.len()]; v.len()];
    for i in 0..v.len() { m[i][i] = v[i]; }
    Ok(json!(m))
}

fn vecmul(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let m = matrix(&args[0])?;
    let v = vector(&args[1])?;
    if m[0].len() != v.len() { return Err("SHAPE"); }
    finite_vec(m.iter().map(|row| row.iter().zip(v.iter()).map(|(a, b)| a * b).sum()).collect())
}

fn outer(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let a = vector(&args[0])?; let b = vector(&args[1])?;
    if a.is_empty() || b.is_empty() || a.len().saturating_mul(b.len()) > MAX_ELEMS { return Err("LIMIT"); }
    finite_matrix(a.into_iter().map(|x| b.iter().map(|y| x * y).collect()).collect())
}

fn inverse_op(args: &[Value]) -> Result<Value, &'static str> {
    let a = one_matrix(args)?;
    let n = a.len();
    if n != a[0].len() || n > 32 { return Err("SHAPE"); }
    let mut aug = vec![vec![0.0; n * 2]; n];
    for i in 0..n { for j in 0..n { aug[i][j] = a[i][j]; } aug[i][n + i] = 1.0; }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n { if aug[r][i].abs() > aug[pivot][i].abs() { pivot = r; } }
        if aug[pivot][i].abs() <= 1e-15 { return Err("SINGULAR"); }
        if pivot != i { aug.swap(i, pivot); }
        let p = aug[i][i];
        for c in 0..(2 * n) { aug[i][c] /= p; }
        for r in 0..n {
            if r == i { continue; }
            let f = aug[r][i];
            for c in 0..(2 * n) { aug[r][c] -= f * aug[i][c]; }
        }
    }
    let out: Vec<Vec<f64>> = (0..n).map(|i| aug[i][n..].to_vec()).collect();
    finite_matrix(out)
}

fn finite(x: f64) -> Result<Value, &'static str> { if x.is_finite() { Ok(json!(x)) } else { Err("NONFINITE") } }
fn finite_vec(v: Vec<f64>) -> Result<Value, &'static str> { if v.iter().all(|x| x.is_finite()) { Ok(json!(v)) } else { Err("NONFINITE") } }
fn finite_matrix(m: Vec<Vec<f64>>) -> Result<Value, &'static str> { if m.iter().flatten().all(|x| x.is_finite()) { Ok(json!(m)) } else { Err("NONFINITE") } }
