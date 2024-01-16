use crate::lcs::LcsStrResult::{Added, Both, Deleted};
use ndarray::{Array2, ArrayView1};
use std::cmp::max;
use std::ops::Index;
use unicode_segmentation::UnicodeSegmentation;

pub enum LcsStrResult {
    Both(String),
    Deleted(String),
    Added(String),
}

fn split(text: &str) -> Vec<String> {
    // input: Lorem ipsum dolor sit amet
    // output: ["Lorem", " ", "ipsum", " ", "dolor", " ", "sit", " ", "amet"]
    if text.len() == 0 {
        return vec![];
    }

    let mut result = vec![];
    let mut current = String::from("");
    for c in text.chars() {
        let is_whitespace = c.is_whitespace();
        if is_whitespace {
            if current.len() > 0 {
                result.push(current.clone());
                current = String::from("");
            }
            result.push(c.to_string());
        } else {
            current.push(c);
        }
    }
    if current.len() > 0 {
        result.push(current);
    }
    result
}

pub fn str_lcs_by_words<'a>(before: &'a str, after: &'a str) -> Vec<LcsStrResult> {
    let before: Vec<_> = split(before);
    let after: Vec<_> = split(after);
    let (before_map, after_map) = vec_lcs(&before, &after);
    let len_merged = before_map.len();

    if len_merged == 0 {
        return vec![];
    }

    let mut result = vec![];
    let mut i = 0;
    while i < len_merged {
        let mut current: String = String::from("");
        match (before_map[i], after_map[i]) {
            (Some(_), Some(_)) => {
                while let (Some(i_before), Some(_)) = (before_map[i], after_map[i]) {
                    current.push_str(before[i_before].as_str());
                    i += 1;
                    if i >= len_merged {
                        break;
                    }
                }
                result.push(Both(current));
            }
            (Some(_), None) => {
                while let (Some(i_before), None) = (before_map[i], after_map[i]) {
                    current.push_str(before[i_before].as_str());
                    i += 1;
                    if i >= len_merged {
                        break;
                    }
                }
                result.push(Deleted(current));
            }
            (None, Some(_)) => {
                while let (None, Some(i_after)) = (before_map[i], after_map[i]) {
                    current.push_str(after[i_after].as_str());
                    i += 1;
                    if i >= len_merged {
                        break;
                    }
                }
                result.push(Added(current));
            }
            (None, None) => {
                unreachable!();
            }
        }
    }
    result
}

pub fn str_lcs_by_graphemes<'a>(before: &'a str, after: &'a str) -> Vec<LcsStrResult> {
    let before: Vec<&str> = before.graphemes(true).collect();
    let after: Vec<&str> = after.graphemes(true).collect();
    let (before_map, after_map) = vec_lcs(&before, &after);
    let len_merged = before_map.len();

    if len_merged == 0 {
        return vec![];
    }

    let mut result = vec![];
    let mut i = 0;
    while i < len_merged {
        let mut current: String = String::from("");
        match (before_map[i], after_map[i]) {
            (Some(_), Some(_)) => {
                while let (Some(i_before), Some(_)) = (before_map[i], after_map[i]) {
                    current.push_str(before[i_before]);
                    i += 1;
                    if i >= len_merged {
                        break;
                    }
                }
                result.push(Both(current));
            }
            (Some(_), None) => {
                while let (Some(i_before), None) = (before_map[i], after_map[i]) {
                    current.push_str(before[i_before]);
                    i += 1;
                    if i >= len_merged {
                        break;
                    }
                }
                result.push(Deleted(current));
            }
            (None, Some(_)) => {
                while let (None, Some(i_after)) = (before_map[i], after_map[i]) {
                    current.push_str(after[i_after]);
                    i += 1;
                    if i >= len_merged {
                        break;
                    }
                }
                result.push(Added(current));
            }
            (None, None) => {
                unreachable!();
            }
        }
    }
    result
}

pub fn vec_lcs<'a, T: Eq>(
    before: &'a Vec<T>,
    after: &'a Vec<T>,
) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
    lcs_core(before, before.len(), after, after.len())
}

pub fn array_lcs<'a, T: Eq>(
    before: &'a ArrayView1<&'a T>,
    after: &'a ArrayView1<&'a T>,
) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
    lcs_core(before, before.shape()[0], after, after.shape()[0])
}

pub fn lcs_core<'a, Container: Index<usize>>(
    before: &'a Container,
    before_size: usize,
    after: &'a Container,
    after_size: usize,
) -> (Vec<Option<usize>>, Vec<Option<usize>>)
where
    Container::Output: Eq,
{
    let mut memoization: Array2<usize> = Array2::zeros((before_size + 1, after_size + 1));
    for i in 1..before_size + 1 {
        for j in 1..after_size + 1 {
            if before[i - 1] == after[j - 1] {
                memoization[[i, j]] = memoization[[i - 1, j - 1]] + 1;
            } else {
                memoization[[i, j]] = max(memoization[[i - 1, j]], memoization[[i, j - 1]]);
            }
        }
    }

    let mut i = before_size;
    let mut j = after_size;
    let len_merged: usize = before_size + after_size - memoization[[before_size, after_size]];
    let mut map_to_before = vec![None; len_merged];
    let mut map_to_after = vec![None; len_merged];

    for k in (0..len_merged).rev() {
        if i > 0 && memoization[[i, j]] == memoization[[i - 1, j]] {
            map_to_before[k] = Some(i - 1);
            i -= 1;
        } else if j > 0 && memoization[[i, j]] == memoization[[i, j - 1]] {
            map_to_after[k] = Some(j - 1);
            j -= 1;
        } else if i > 0 && j > 0 && memoization[[i - 1, j - 1]] + 1 == memoization[[i, j]] {
            map_to_before[k] = Some(i - 1);
            map_to_after[k] = Some(j - 1);
            i -= 1;
            j -= 1;
        } else {
            unreachable!();
        }
    }

    (map_to_before, map_to_after)
}
