use crate::error;

pub(crate) fn require_not_empty(value: &String, element: &str) -> Result<(), error::ErrorResponse> {
    if value.trim().is_empty() {
        Err(error::ErrorResponse { code: "empty_value".into(), message: format!("{} cannot be empty or whitespace", element) })
    } else {
        Ok(())
    }
}

pub(crate) fn vec_require_not_empty<T>(items: &[T],  element: &str) -> Result<(), error::ErrorResponse> {
    if items.is_empty() {
        Err(error::ErrorResponse { code: "empty_list".into(), message: format!("{} must contain at least one item", element) })
    } else {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_not_empty_is_empty() {
        let result = require_not_empty(&"".to_string(), "field_name");
    
        assert!(matches!(
            result,
            Err(error::ErrorResponse { code, .. }) if code == "empty_value"
        ));
    }

    #[test]
    fn test_require_not_empty_has_whitespace() {
        let result = require_not_empty(&"   ".to_string(), "field_name");
    
        assert!(matches!(
            result,
            Err(error::ErrorResponse { code, .. }) if code == "empty_value"
        ));
    }

    #[test]
    fn test_require_not_empty_is_not_empty() {
        let result = require_not_empty(&"abc".to_string(), "field_name");
    
        assert!(result.is_ok());
    }

     #[test]
    fn vec_require_not_empty_returns_ok_for_non_empty_vec() {
        let items = vec![1, 2, 3];

        let result = vec_require_not_empty(&items, "numbers");

        assert!(result.is_ok());
    }

    #[test]
    fn vec_require_not_empty_returns_err_for_empty_vec() {
        let items: Vec<i32> = vec![];

        let result = vec_require_not_empty(&items, "numbers");

        assert!(result.is_err());
    }    
}