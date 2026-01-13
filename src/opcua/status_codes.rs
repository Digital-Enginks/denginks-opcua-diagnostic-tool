//! Status Code translation helper
//!
//! Maps OPC-UA StatusCode values to friendly names/descriptions.

use opcua::types::StatusCode;

/// Translate a StatusCode into a user-friendly string
pub fn translate_status_code(code: StatusCode) -> String {
    crate::utils::status_codes::translate_status_code(code.bits())
}
