//! OPC-UA Status Code translation
//!
//! Converts OPC-UA status codes to human-readable messages

pub fn translate_status_code(code: u32) -> String {
    // First try exact match
    let msg = match code {
        // Good status codes
        0x00000000 => Some("Good"),
        0x00000001 => Some("Good - Clamped"),
        0x00000002 => Some("Good - More Data"),
        0x00000003 => Some("Good - Communication Event"),
        0x00000004 => Some("Good - Shutdown Event"),
        0x00000005 => Some("Good - Call Again"),
        0x00000006 => Some("Good - No Data"),
        
        // Uncertain status codes
        0x40000000 => Some("Uncertain"),
        0x40010000 => Some("Uncertain - Initial Value"),
        0x40020000 => Some("Uncertain - Sensor Calibration"),
        0x40030000 => Some("Uncertain - Engineering Units Exceeded"),
        0x40040000 => Some("Uncertain - Sub Normal"),
        0x40050000 => Some("Uncertain - Last Usable Value"),
        0x40A50000 => Some("Uncertain - Data Sub Normal"),
        
        // Bad status codes
        0x80000000 => Some("Bad - Unexpected Error"),
        0x80010000 => Some("Bad - Internal Error"),
        0x80020000 => Some("Bad - Out Of Memory"),
        0x80030000 => Some("Bad - Resource Unavailable"),
        0x80040000 => Some("Bad - Communication Error"),
        0x80050000 => Some("Bad - Encoding Error"),
        0x80060000 => Some("Bad - Decoding Error"),
        0x80070000 => Some("Bad - Encoding Limits Exceeded"),
        0x80080000 => Some("Bad - Request Too Large"),
        0x80090000 => Some("Bad - Response Too Large"),
        0x800A0000 => Some("Bad - Unknown Response"),
        0x800B0000 => Some("Bad - Timeout"),
        0x800C0000 => Some("Bad - Service Unsupported"),
        0x800D0000 => Some("Bad - Shutdown"),
        0x800E0000 => Some("Bad - Server Not Connected"),
        0x800F0000 => Some("Bad - Server Halted"),
        0x80100000 => Some("Bad - Nothing To Do"),
        0x80110000 => Some("Bad - Too Many Operations"),
        0x80120000 => Some("Bad - Too Many Monitored Items"),
        0x80130000 => Some("Bad - Data Type ID Unknown"),
        0x80140000 => Some("Bad - Certificate Invalid"),
        0x80150000 => Some("Bad - Security Checks Failed"),
        0x80160000 => Some("Bad - Certificate Time Invalid"),
        0x80170000 => Some("Bad - Certificate Issuer Time Invalid"),
        0x80180000 => Some("Bad - Certificate Host Name Invalid"),
        0x80190000 => Some("Bad - Certificate URI Invalid"),
        0x801A0000 => Some("Bad - Certificate Use Not Allowed"),
        0x801B0000 => Some("Bad - Certificate Issuer Use Not Allowed"),
        0x801C0000 => Some("Bad - Certificate Untrusted"),
        0x801D0000 => Some("Bad - Certificate Revocation Unknown"),
        0x801E0000 => Some("Bad - Certificate Issuer Revocation Unknown"),
        0x801F0000 => Some("Bad - Certificate Revoked"),
        0x80200000 => Some("Bad - Certificate Issuer Revoked"),
        0x80210000 => Some("Bad - Certificate Chain Incomplete"),
        0x80220000 => Some("Bad - User Access Denied"),
        0x80230000 => Some("Bad - Identity Token Invalid"),
        0x80240000 => Some("Bad - Identity Token Rejected"),
        0x80250000 => Some("Bad - Secure Channel ID Invalid"),
        0x80260000 => Some("Bad - Invalid Timestamp"),
        0x80270000 => Some("Bad - Nonce Invalid"),
        0x80280000 => Some("Bad - Session ID Invalid"),
        0x80290000 => Some("Bad - Session Closed"),
        0x802A0000 => Some("Bad - Session Not Activated"),
        0x802B0000 => Some("Bad - Subscription ID Invalid"),
        0x80890000 => Some("Bad - Node ID Invalid"),
        0x808A0000 => Some("Bad - Node ID Unknown"),
        0x808B0000 => Some("Bad - Attribute ID Invalid"),
        0x808C0000 => Some("Bad - Index Range Invalid"),
        0x808D0000 => Some("Bad - Index Range No Data"),
        0x808E0000 => Some("Bad - Data Encoding Invalid"),
        0x808F0000 => Some("Bad - Data Encoding Unsupported"),
        0x80900000 => Some("Bad - Not Readable"),
        0x80910000 => Some("Bad - Not Writable"),
        0x80920000 => Some("Bad - Out Of Range"),
        0x80930000 => Some("Bad - Not Supported"),
        0x80940000 => Some("Bad - Not Found"),
        0x80950000 => Some("Bad - Object Deleted"),
        0x80960000 => Some("Bad - Not Implemented"),
        0x80970000 => Some("Bad - Monitoring Mode Invalid"),
        0x80980000 => Some("Bad - Monitored Item ID Invalid"),
        0x80990000 => Some("Bad - Monitored Item Filter Not Supported"),
        0x809A0000 => Some("Bad - Monitored Item Filter Unsupported"),
        0x809B0000 => Some("Bad - Filter Not Allowed"),
        0x809C0000 => Some("Bad - Structure Missing"),
        0x809D0000 => Some("Bad - Event Filter Invalid"),
        0x809E0000 => Some("Bad - Content Filter Invalid"),
        0x809F0000 => Some("Bad - Filter Operator Invalid"),
        0x80A00000 => Some("Bad - Filter Operator Unsupported"),
        0x80A10000 => Some("Bad - Filter Operand Count Mismatch"),
        0x80A20000 => Some("Bad - Filter Operand Invalid"),
        0x80A30000 => Some("Bad - Filter Element Invalid"),
        0x80A40000 => Some("Bad - Filter Literal Invalid"),
        0x80A80000 => Some("Bad - Continuation Point Invalid"),
        0x80A90000 => Some("Bad - No Continuation Points"),
        0x80AA0000 => Some("Bad - Reference Type ID Invalid"),
        0x80AB0000 => Some("Bad - Browse Direction Invalid"),
        0x80AC0000 => Some("Bad - Node Not In View"),
        0x80AD0000 => Some("Bad - Numeric Overflow"),
        0x80AE0000 => Some("Bad - Server URI Invalid"),
        0x80AF0000 => Some("Bad - Server Name Missing"),
        0x80B00000 => Some("Bad - Discovery URL Missing"),
        0x80B10000 => Some("Bad - Semaphore File Missing"),
        0x80B20000 => Some("Bad - Request Type Invalid"),
        0x80B30000 => Some("Bad - Security Mode Rejected"),
        0x80B40000 => Some("Bad - Security Policy Rejected"),
        0x80B50000 => Some("Bad - Too Many Sessions"),
        0x80B60000 => Some("Bad - User Signature Invalid"),
        0x80B70000 => Some("Bad - Application Signature Invalid"),
        0x80B80000 => Some("Bad - No Valid Certificates"),
        0x80B90000 => Some("Bad - Identity Change Not Supported"),
        0x80BD0000 => Some("Bad - Request Cancelled By Request"),
        0x80BE0000 => Some("Bad - Parent Node ID Invalid"),
        0x80BF0000 => Some("Bad - Reference Not Allowed"),
        0x80C10000 => Some("Bad - Node ID Rejected"),
        0x80C20000 => Some("Bad - Node ID Exists"),
        0x80C30000 => Some("Bad - Node Class Invalid"),
        0x80C40000 => Some("Bad - Browse Name Invalid"),
        0x80C50000 => Some("Bad - Browse Name Duplicated"),
        0x80C60000 => Some("Bad - Node Attributes Invalid"),
        0x80C70000 => Some("Bad - Type Definition Invalid"),
        0x80C80000 => Some("Bad - Source Node ID Invalid"),
        0x80C90000 => Some("Bad - Target Node ID Invalid"),
        0x80CA0000 => Some("Bad - Duplicate Reference Not Allowed"),
        0x80CB0000 => Some("Bad - Invalid Self Reference"),
        0x80CC0000 => Some("Bad - Reference Local Only"),
        0x80CD0000 => Some("Bad - No Delete Rights"),
        _ => None,
    };

    if let Some(m) = msg {
        return m.to_string();
    }
    
    // Determine severity from high bits
    let severity = match code >> 30 {
        0 => "Good",
        1 => "Uncertain",
        2 | 3 => "Bad",
        _ => "Unknown",
    };
    
    format!("{} (0x{:08X})", severity, code)
}

#[allow(dead_code)]
pub fn status_code_color(code: u32) -> [u8; 3] {
    match code >> 30 {
        0 => [0, 200, 0],       // Good - Green
        1 => [255, 200, 0],     // Uncertain - Yellow
        _ => [255, 50, 50],     // Bad - Red
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_good() {
        assert_eq!(translate_status_code(0x00000000), "Good");
    }

    #[test]
    fn test_translate_bad_certificate() {
        assert_eq!(translate_status_code(0x801C0000), "Bad - Certificate Untrusted");
    }

    #[test]
    fn test_translate_unknown() {
        let result = translate_status_code(0x80FF0000);
        assert!(result.contains("Bad"));
        assert!(result.contains("0x80FF0000"));
    }
}
