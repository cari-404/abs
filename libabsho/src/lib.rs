#[no_mangle]
#[inline(always)]
pub extern "C" fn my_function(x: i32) -> i32 {
    x * 2
}
#[no_mangle]
#[inline(always)]
pub extern "C"  fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
