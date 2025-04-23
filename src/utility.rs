
/// returns length of number(`n`) in `base`:
/// # Example
/// * `13` = `121_base3` len is `3`  
/// * `13` = `23_base5` len is `2`
/// * `13` = `D_base16` len is `1`
/// 
/// len of `0` always is `0`
pub fn length_in_base(mut n: usize, base: usize) -> u32 {
    let mut len = 0;
    while n > 0 {
        n /= base;
        len += 1;
    }
    len
}

/// return valid index for current continuation of `MarkleTree` level:
/// # Example
/// ```txt
/// 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | ...                          <-- valid indexes
/// 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 |  9 10 11 |  9 10 11 || ...   <-- result
/// 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 14 | 15 16 17 || ...   <-- index
/// ```
/// 
/// return index is decided by last arity-digit that less in `index` than in `max_valid_index`:
/// ```txt
/// 121202
///     ^  --> 111102
/// 111111
/// ```
/// <br/>
/// 
/// ```txt
/// 120202
///   ^    --> 110202
/// 111111
/// ```
pub fn get_pad_index(mut index: usize, mut max_valid_index: usize, arity: usize) -> usize {
    if index <= max_valid_index { return index }

    let mut shift = 1;
    
    let mut tail = 0;
    let mut cur_tail = 0;
    
    let mut head = 0;

    while index > 0 {
        let tail_digit = index % arity;
        let head_digit = max_valid_index % arity;

        cur_tail += tail_digit * shift;
        head += head_digit * shift;

        if tail_digit < head_digit {
            tail = cur_tail;
            head = 0;
        }

        shift *= arity;
        index /= arity;
        max_valid_index /= arity;
    }

    head + tail
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MtLvl;

    #[test]
    fn length_in_base_test() {
        assert_eq!(length_in_base(0, 3), 0);
        assert_eq!(length_in_base(1, 3), 1);
        assert_eq!(length_in_base(2, 3), 1);
        assert_eq!(length_in_base(3, 3), 2);
        assert_eq!(length_in_base(5, 3), 2);
        assert_eq!(length_in_base(8, 3), 2);
        assert_eq!(length_in_base(9, 3), 3);
        assert_eq!(length_in_base(10, 3), 3);
        assert_eq!(length_in_base(26, 3), 3);
        assert_eq!(length_in_base(27, 3), 4);
        assert_eq!(length_in_base(28, 3), 4);
        assert_eq!(length_in_base(80, 3), 4);
        assert_eq!(length_in_base(85, 3), 5);

        assert_eq!(length_in_base(13, 3), 3);
        assert_eq!(length_in_base(13, 5), 2);
        assert_eq!(length_in_base(13, 10), 2);
        assert_eq!(length_in_base(13, 16), 1);
    }

    #[cfg(test)]
    #[test]
    fn get_pad_index_test() {
        // 0 1 2 3 4 | 5 6 7 ...                        <-- (max_valid_index = 7)
        // 0 1 2 3 4 | 5 6 7 7 7 |  5  6  7  7  7 | ... <-- result
        // 0 1 2 3 4 | 5 6 7 8 9 | 10 11 12 13 14 | ... <-- index
        assert_eq!(get_pad_index(14, 7, 5), 7);
        assert_eq!(get_pad_index(13, 7, 5), 7);
        assert_eq!(get_pad_index(7, 14, 5), 7);

        assert_eq!(get_pad_index(9, 7, 5), 7);
        assert_eq!(get_pad_index(10, 7, 5), 5);
        assert_eq!(get_pad_index(11, 7, 5), 6);
        assert_eq!(get_pad_index(7, 11, 5), 7);

        // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | ...      <-- (max_valid_index = 11)
        // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 |  9 10 11 |  9 10 11 || ...      <-- result
        // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 14 | 15 16 17 || ...      <-- index
        for j in 0..3 {
            for i in 0..3 {
                assert_eq!(get_pad_index(12 + i + j * 3, 11, 3), 9 + i);
            }
        }
        assert_eq!(get_pad_index(26, 11, 3), 11);
        assert_eq!(get_pad_index(18, 11, 3), 9);
        assert_eq!(get_pad_index(19, 11, 3), 10);
        assert_eq!(get_pad_index(20, 11, 3), 11);
        assert_eq!(get_pad_index(21, 11, 3), 9);
        assert_eq!(get_pad_index(22, 11, 3), 10);
        assert_eq!(get_pad_index(23, 11, 3), 11);

        for k in 1..=2 {
            for i in 0..=11 {
                assert_eq!(get_pad_index(27*k + i, 11, 3), i);
            }
            for j in 0..2 {
                for i in 0..3 {
                    assert_eq!(get_pad_index(27*k + 12 + i + j * 3, 11, 3), 9 + i);
                }
            }
        }

        // ... | 24 25 26 ||| 27 (27 + 1) (27 + 2) | (27 + 3) (27 + 4) (27 + 5) ...  <-- (max_valid_index = 27 + 6 - 1)
        // eq
        //                ||| 27 28 29 | 30 31 32 | 30 31 32 || 27 28 29 | 30 31 32 | 30 31 32 || ...
        //                ||| 54 55 56 | 57 58 59 | 60 61 62 || 63 64 65 | 66 67 68 | 69 70 71 || ...
        assert_eq!(get_pad_index(27 * 2 + 6 - 1, 27 + 6 - 1, 3), 27 + 6 - 1);
        for i in 0..6 {
            assert_eq!(get_pad_index(27 * 2 + i, 27 + 6 - 1, 3), 27 + i);
            assert_eq!(get_pad_index(27 * 2 + 9 + i, 27 + 6 - 1, 3), 27 + i);
        }
        for i in 0..3 {
            assert_eq!(get_pad_index(27 * 2 + 6 + i, 27 + 6 - 1, 3), 27 + 3 + i);
            assert_eq!(get_pad_index(27 * 2 + 9 + 6 + i, 27 + 6 - 1, 3), 27 + 3 + i);
        }

        // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 __ | ..                <-- valid indexes
        // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 13 | 12 13 13 || ...   <-- result
        // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 14 | 15 16 17 || ...   <-- index
        let result = "0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 13 | 12 13 13 || 9 10 11 | 12 13 13 | 12 13 13";
        let result: Vec<usize> = crate::tests::to_vec(result);
        let indexes = (0..27).into_iter().collect::<Vec<_>>();
        assert_eq!(indexes.len(), result.len());
        for index in indexes {
            assert_eq!(get_pad_index(index, 13, 3), result[index], "index is {index}");
        }
        
        let a: Vec<usize> = (0..=364).collect();
        let result = MtLvl::<_, 3>::vec_continuation(a.clone());
        for k in 0..3 {
            for index in 0..result.len() {
                assert_eq!(get_pad_index(index + k * result.len(), a.len() - 1, 3), result[index]);
            }
        }
        
        let a: Vec<usize> = (0..=363).collect();
        let result = MtLvl::<_, 5>::vec_continuation(a.clone());
        for k in 0..3 {
            for index in 0..result.len() {
                assert_eq!(get_pad_index(index + k * result.len(), a.len() - 1, 5), result[index]);
            }
        }
    }
}