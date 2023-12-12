use crate::sqlite::Type::Text;
use crate::sqlite::{get_tables, read_table_as_2d_array, DatumWithType, Type};
use anyhow::Result;
use lcs::*;
use ndarray::prelude::*;
use rust_xlsxwriter::{Color, Format, FormatBorder, Worksheet};

fn diff<'a>(
    before: &str,
    after: &str,
    green: &'a Format,
    red: &'a Format,
    black: &'a Format,
) -> (Vec<(&'a Format, String)>, Vec<(&'a Format, String)>) {
    let has_whitespace = before.contains(' ') || after.contains(' ');
    let lcs_results = {
        if has_whitespace {
            str_lcs_by_words(before, after)
        } else {
            str_lcs_by_graphemes(before, after)
        }
    };

    let mut result = (vec![], vec![]);

    for lcs_result in lcs_results {
        match lcs_result {
            LcsStrResult::Both(content) => {
                result.0.push((black, content.clone()));
                result.1.push((black, content));
            }
            LcsStrResult::Deleted(content) => {
                result.0.push((red, content));
            }
            LcsStrResult::Added(content) => {
                result.1.push((green, content));
            }
        }
    }

    result
}

pub fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let conn1 = rusqlite::Connection::open(&args[1])?;
    let conn2 = rusqlite::Connection::open(&args[2])?;

    let tables1 = get_tables(&conn1)?;
    let table1 = tables1.first().unwrap();
    let tables2 = get_tables(&conn2)?;
    let table2 = tables2.first().unwrap();

    let before = read_table_as_2d_array(&conn1, table1)?;
    let after = read_table_as_2d_array(&conn2, table2)?;
    let map = compare(&before, &after);

    let mut book = rust_xlsxwriter::Workbook::new();
    let sheet = book.add_worksheet();

    let (merged_row_len, merged_column_len) =
        (map.row_map_to_before.len(), map.column_map_to_before.len());

    let (i0, j0) = (0, 0);
    let (i1, j1) = (0, 1 + merged_column_len as u16);
    for i in 0..merged_row_len {
        for j in 0..merged_column_len {
            match (
                map.row_map_to_before[i],
                map.column_map_to_before[j],
                map.row_map_to_after[i],
                map.column_map_to_after[j],
            ) {
                (Some(ib), Some(jb), Some(ia), Some(ja)) => {
                    if before[[ib, jb]] == after[[ia, ja]] {
                        write_to_sheet(
                            sheet,
                            i0 + i as u32,
                            j0 + j as u16,
                            &before[[ib, jb]],
                            Color::White,
                        );
                        write_to_sheet(
                            sheet,
                            i1 + i as u32,
                            j1 + j as u16,
                            &after[[ia, ja]],
                            Color::White,
                        );
                    } else if before[[ib, jb]].datum_type == Text
                        && after[[ia, ja]].datum_type == Text
                    {
                        let before_text = std::str::from_utf8(&before[[ib, jb]].datum).unwrap();
                        let after_text = std::str::from_utf8(&after[[ia, ja]].datum).unwrap();

                        let black = Format::new().set_font_color(Color::Black);
                        let red = Format::new().set_font_color(Color::Red);
                        let green = Format::new().set_font_color(Color::Green);
                        let bg = Format::new().set_background_color(Color::Yellow);
                        let (rich_input_1, rich_input_2) =
                            diff(before_text, after_text, &green, &red, &black);
                        sheet
                            .write_rich_string_with_format(
                                i0 + i as u32,
                                j0 + j as u16,
                                rich_input_1
                                    .iter()
                                    .map(|(f, s)| (*f, &**s))
                                    .collect::<Vec<(&Format, &str)>>()
                                    .as_slice(),
                                &bg,
                            )
                            .expect("too many chars");
                        sheet
                            .write_rich_string_with_format(
                                i1 + i as u32,
                                j1 + j as u16,
                                rich_input_2
                                    .iter()
                                    .map(|(f, s)| (*f, &**s))
                                    .collect::<Vec<(&Format, &str)>>()
                                    .as_slice(),
                                &bg,
                            )
                            .expect("too many chars 2");
                    } else {
                        write_to_sheet(
                            sheet,
                            i0 + i as u32,
                            j0 + j as u16,
                            &before[[ib, jb]],
                            Color::Yellow,
                        );
                        write_to_sheet(
                            sheet,
                            i1 + i as u32,
                            j1 + j as u16,
                            &after[[ia, ja]],
                            Color::Yellow,
                        );
                    }
                }
                (Some(ib), Some(jb), _, _) => {
                    write_to_sheet(
                        sheet,
                        i0 + i as u32,
                        j0 + j as u16,
                        &before[[ib, jb]],
                        Color::Red,
                    );
                    write_gray_blank(sheet, i1 + i as u32, j1 + j as u16);
                }
                (_, _, Some(ia), Some(ja)) => {
                    write_gray_blank(sheet, i0 + i as u32, j0 + j as u16);
                    write_to_sheet(
                        sheet,
                        i1 + i as u32,
                        j1 + j as u16,
                        &after[[ia, ja]],
                        Color::Green,
                    );
                }
                (None, _, None, _) => {
                    unreachable!();
                }
                (_, None, _, None) => {
                    unreachable!();
                }
                _ => {
                    // ex) row 삭제 column 추가면 여기로 타는게 가능
                    //  + 가 추가, -가 삭제라 했을 때 아래와 같은 그림
                    // 이 unified 그림에서 (1,1)은 before에도 after에도 없다.
                    // 그래서 둘 다 gray blank로 그린다.
                    // |  a  | + c |
                    // | - b | + - |
                    write_gray_blank(sheet, i0 + i as u32, j0 + j as u16);
                    write_gray_blank(sheet, i1 + i as u32, j1 + j as u16);
                }
            }
        }
    }

    let dir = std::env::temp_dir();
    let rand = rand::random::<u32>();
    let filename_1 = std::path::Path::file_name(std::path::Path::new(&args[1])).unwrap();
    let filename_2 = std::path::Path::file_name(std::path::Path::new(&args[2])).unwrap();
    let file_name = format!(
        "{}-{}-{}.xlsx",
        filename_1.to_str().unwrap(),
        filename_2.to_str().unwrap(),
        rand
    );
    let path = dir.join(file_name);
    book.save(path.clone())
        .expect(format!("failed to save {}", path.display()).as_str());
    println!("{}", path.display());
    Ok(())
}

fn write_gray_blank(sheet: &mut Worksheet, row: u32, column: u16) {
    let format = Format::new().set_background_color(Color::Gray);
    sheet.write_blank(row, column, &format).unwrap();
}

mod sqlite;

fn compare<T>(before: &Array2<T>, after: &Array2<T>) -> IndexMap
where
    T: Eq,
{
    let before_first_row: ArrayView1<_> = before.slice(s![.., 0]);
    let after_first_row: ArrayView1<_> = after.slice(s![.., 0]);
    let before_first_column: ArrayView1<_> = before.slice(s![0, ..]);
    let after_first_column: ArrayView1<_> = after.slice(s![0, ..]);

    let (row_map_to_before, row_map_to_after) = lcs_core(
        &before_first_row,
        before_first_row.shape()[0],
        &after_first_row,
        after_first_row.shape()[0],
    );
    let (column_map_to_before, column_map_to_after) = lcs_core(
        &before_first_column,
        before_first_column.shape()[0],
        &after_first_column,
        after_first_column.shape()[0],
    );

    IndexMap {
        row_map_to_before,
        row_map_to_after,
        column_map_to_before,
        column_map_to_after,
    }
}

struct IndexMap {
    row_map_to_before: Vec<Option<usize>>,
    row_map_to_after: Vec<Option<usize>>,
    column_map_to_before: Vec<Option<usize>>,
    column_map_to_after: Vec<Option<usize>>,
}

fn write_to_sheet(
    sheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: &DatumWithType,
    color: Color,
) {
    let format = Format::new()
        .set_background_color(color)
        .set_border(FormatBorder::Thick);
    match value {
        DatumWithType {
            datum,
            datum_type: Type::Integer,
        } => {
            sheet
                .write_number_with_format(
                    row,
                    column,
                    i64::from_le_bytes(datum.as_slice().try_into().unwrap()) as f64,
                    &format,
                )
                .unwrap();
        }
        DatumWithType {
            datum,
            datum_type: Type::Real,
        } => {
            sheet
                .write_number_with_format(
                    row,
                    column,
                    f64::from_le_bytes(datum.as_slice().try_into().unwrap()),
                    &format,
                )
                .unwrap();
        }
        DatumWithType {
            datum,
            datum_type: Type::Text,
        } => {
            sheet
                .write_string_with_format(row, column, std::str::from_utf8(datum).unwrap(), &format)
                .unwrap();
        }
        DatumWithType {
            datum,
            datum_type: Type::Blob,
        } => {
            sheet
                .write_string_with_format(row, column, std::str::from_utf8(datum).unwrap(), &format)
                .unwrap();
        }
        DatumWithType {
            datum,
            datum_type: Type::Null,
        } => {
            sheet
                .write_string_with_format(row, column, std::str::from_utf8(datum).unwrap(), &format)
                .unwrap();
        }
    }
}

mod lcs;

#[cfg(test)]
mod tests {
    use crate::{lcs, main};

    #[test]
    fn asdf() {
        main();
    }
}
