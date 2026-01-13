use opcua::types::StatusCode;


pub fn translate_status_code(code: StatusCode) -> String {
    crate::utils::status_codes::translate_status_code(code.bits())
}

#[cfg(test)]
mod tests {
    use super::*;
    use opcua::types::StatusCode;

    #[test]
    fn test_wrapper() {
        let code = StatusCode::Good;
        assert_eq!(translate_status_code(code), "Good");
    }
}