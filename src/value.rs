#[derive(Debug, Clone)]
pub enum Value {
    Num(f64),
    Str(String),
    Null
}
