use thiserror::Error;

#[derive(Error, Debug)]
pub enum Dist2MstError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at row {row}, col {col}: {msg}")]
    Parse { row: usize, col: usize, msg: String },

    #[error("Matrix is not square: {rows} rows x {cols} cols")]
    NotSquare { rows: usize, cols: usize },

    #[error("Empty matrix")]
    EmptyMatrix,

    #[error("{0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, Dist2MstError>;
