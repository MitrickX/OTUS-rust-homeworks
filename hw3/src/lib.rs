pub fn get_pair_mut_ref_elem(pair: &mut (i32, i32), is_second: bool) -> &mut i32 {
    if !is_second {
        &mut pair.0
    } else {
        &mut pair.1
    }
}

pub fn get_slice_mut_ref_elem(slice: &mut [i32], idx: usize) -> &mut i32 {
    &mut slice[idx]
}

pub fn get_slice_end_ref_elem(slice: &[i32], n: usize) -> &i32 {
    let idx = slice.len() - n - 1;
    &slice[idx]
}

pub fn split_slice_2_parts(slice: &[i32], n: usize) -> (&[i32], &[i32]) {
    (&slice[0..n], &slice[n..])
}

pub fn split_slice_4_parts(slice: &[i32]) -> (&[i32], &[i32], &[i32], &[i32]) {
    let n = slice.len();
    let k = n / 4;

    // размеры слайсов
    let (mut s1, mut s2, mut s3) = (k, k, k);

    if n % 4 == 1 {
        s1 += 1;
    } else if n % 4 == 2 {
        s1 += 1;
        s2 += 1;
    } else if n % 4 == 3 {
        s1 += 1;
        s2 += 1;
        s3 += 1;
    }

    (
        &slice[0..s1],
        &slice[s1..s1 + s2],
        &slice[s1 + s2..s1 + s2 + s3],
        &slice[s1 + s2 + s3..],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_pair_mut_ref_elem_works() {
        let mut test_pair = (1, 2);
        let mut_ref_elem = get_pair_mut_ref_elem(&mut test_pair, true);
        *mut_ref_elem = 3;
        assert_eq!(test_pair, (1, 3));

        let mut_ref_elem = get_pair_mut_ref_elem(&mut test_pair, false);
        *mut_ref_elem = 4;
        assert_eq!(test_pair, (4, 3));
    }

    #[test]
    fn get_slice_mut_ref_elem_works() {
        let mut test_slice = [1, 2, 3, 4, 5, 6];
        let n = test_slice.len();
        for i in 0..n {
            let ref_elem = get_slice_mut_ref_elem(&mut test_slice, i);
            *ref_elem += 1;
        }

        assert_eq!(test_slice, [2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn get_slice_end_ref_elem_works() {
        let test_slice = [1, 2, 3, 4];
        let len = test_slice.len();
        for n in len - 1..0 {
            let ref_elem = get_slice_end_ref_elem(&test_slice, n);
            assert_eq!(*ref_elem, test_slice[len - n - 1])
        }
    }

    #[test]
    fn split_slice_2_parts_works() {
        let test_slice = [1, 2, 3];
        let expected: [(&[i32], &[i32]); 4] = [
            (&[], &[1, 2, 3]),
            (&[1], &[2, 3]),
            (&[1, 2], &[3]),
            (&[1, 2, 3], &[]),
        ];
        let len = test_slice.len();
        for k in 0..len + 1 {
            let result = split_slice_2_parts(&test_slice, k);
            assert_eq!(result, expected[k]);
        }
    }

    #[test]
    fn split_slice_4_parts_works() {
        let test_slice: [i32; 0] = [];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[], &[], &[], &[]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);

        let test_slice = [1];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[1], &[], &[], &[]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);

        let test_slice = [1, 2];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[1], &[2], &[], &[]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);

        let test_slice = [1, 2, 3];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[1], &[2], &[3], &[]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);

        let test_slice = [1, 2, 3, 4];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[1], &[2], &[3], &[4]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);

        let test_slice = [1, 2, 3, 4, 5];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[1, 2], &[3], &[4], &[5]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);

        let test_slice = [1, 2, 3, 4, 5, 6];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[1, 2], &[3, 4], &[5], &[6]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);

        let test_slice = [1, 2, 3, 4, 5, 6, 7];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[1, 2], &[3, 4], &[5, 6], &[7]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);

        let test_slice = [1, 2, 3, 4, 5, 6, 7, 8];
        let expected: (&[i32], &[i32], &[i32], &[i32]) = (&[1, 2], &[3, 4], &[5, 6], &[7, 8]);
        let result = split_slice_4_parts(&test_slice);
        assert_eq!(result, expected);
    }
}
