use ndarray::Array2;
use rusqlite::{Connection, DatabaseName, Error};

pub fn get_tables(conn: &Connection) -> Result<Vec<String>, Error> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
    let rows = stmt.query_map::<String, _, _>([], |row| row.get(0))?;
    Ok(rows.map(|r| r.unwrap()).collect())
}

fn get_table_header(conn: &Connection, table_name: &str) -> Result<Vec<ColumnInfo>, Error> {
    let mut result = vec![];
    conn.pragma(Some(DatabaseName::Main), "table_info", table_name, |row| {
        result.push(ColumnInfo {
            name: row.get(1)?,
            order: row.get(0)?,
            is_primary_key: row.get(5)?,
        });
        Ok(())
    })?;
    Ok(result)
}

pub fn read_table_as_2d_array(
    conn: &Connection,
    table_name: &str,
) -> Result<Array2<DatumWithType>, Error> {
    let header = get_table_header(conn, table_name)?;
    let row_len = conn.query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |row| {
        row.get(0).map(|v: usize| v + 1) // + 1 for header
    })?;
    let column_len = header.len();

    let mut result: Array2<DatumWithType> =
        Array2::from_shape_fn((row_len, column_len), |(_, _)| DatumWithType::default());
    for (j, column) in header.into_iter().enumerate() {
        result[[0, j]] = DatumWithType {
            datum: column.name.into_bytes(),
            datum_type: Type::Text,
        };
    }

    let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table_name))?;
    let mut rows = stmt.query([])?;

    let mut i = 1;
    while let Some(row) = rows.next()? {
        for j in 0..column_len {
            let val = row.get_ref(j)?;
            match val {
                rusqlite::types::ValueRef::Null => {}
                rusqlite::types::ValueRef::Integer(val) => {
                    result[[i, j]] = DatumWithType {
                        datum: val.to_le_bytes().to_vec(),
                        datum_type: Type::Integer,
                    };
                }
                rusqlite::types::ValueRef::Real(r) => {
                    result[[i, j]] = DatumWithType {
                        datum: r.to_le_bytes().to_vec(),
                        datum_type: Type::Real,
                    };
                }
                rusqlite::types::ValueRef::Text(s) => {
                    result[[i, j]] = DatumWithType {
                        datum: s.to_vec(),
                        datum_type: Type::Text,
                    };
                }
                rusqlite::types::ValueRef::Blob(b) => {
                    result[[i, j]] = DatumWithType {
                        datum: b.to_vec(),
                        datum_type: Type::Blob,
                    };
                }
            }
        }
        i += 1;
    }
    Ok(result)
}

struct ColumnInfo {
    name: String,
    order: usize,
    is_primary_key: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DatumWithType {
    pub datum: Vec<u8>,
    pub datum_type: Type,
}

impl Default for DatumWithType {
    fn default() -> Self {
        DatumWithType {
            datum: vec![],
            datum_type: Type::Null,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Type {
    Null,
    Integer,
    Real,
    Text,
    Blob,
}
