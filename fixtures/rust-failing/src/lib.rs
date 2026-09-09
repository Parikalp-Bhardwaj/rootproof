pub fn divide(left: i32, right: i32) -> i32 {
    left / right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intentionally_failing_test() {
        assert_eq!(divide(10, 2), 10);
    }
}