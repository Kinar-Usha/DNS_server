//! Response logging structures for DNS queries

use crate::protocol::QueryType;
use crate::protocol::ResultCode;
use std::fmt;

/// Response information logged for each DNS query
#[derive(Clone, Debug, PartialEq)]
pub struct ResponseLog {
    pub domain_name: String,
    pub query_type: QueryType,
    pub response_time_ms: u64,
    pub answer_count: usize,
    pub authority_count: usize,
    pub response_size: usize,
    pub was_cache_hit: bool,
    pub ttl_values: Vec<u32>,
    pub result_code: ResultCode,
}

impl fmt::Display for ResponseLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Domain: {} | Type: {:?} | Time: {}ms | Cache: {} | Answers: {} | Authority: {} | Size: {}B | TTLs: {:?} | Result: {:?}",
            self.domain_name,
            self.query_type,
            self.response_time_ms,
            if self.was_cache_hit { "HIT" } else { "MISS" },
            self.answer_count,
            self.authority_count,
            self.response_size,
            self.ttl_values,
            self.result_code,
        )
    }
}

impl ResponseLog {
    /// Log the response with INFO level
    pub fn log(&self) {
        log::info!("{}", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_log_format() {
        let log = ResponseLog {
            domain_name: "example.com".to_string(),
            query_type: QueryType::A,
            response_time_ms: 50,
            answer_count: 2,
            authority_count: 1,
            response_size: 128,
            was_cache_hit: false,
            ttl_values: vec![3600, 3600],
            result_code: ResultCode::NOERROR,
        };

        let formatted = format!("{}", log);
        assert!(formatted.contains("example.com"));
        assert!(formatted.contains("50ms"));
        assert!(formatted.contains("MISS"));
        assert!(formatted.contains("Answers: 2"));
        assert!(formatted.contains("128B"));
    }

    #[test]
    fn test_response_log_cache_hit() {
        let log = ResponseLog {
            domain_name: "google.com".to_string(),
            query_type: QueryType::A,
            response_time_ms: 2,
            answer_count: 4,
            authority_count: 0,
            response_size: 256,
            was_cache_hit: true,
            ttl_values: vec![3600, 3600, 3600, 3600],
            result_code: ResultCode::NOERROR,
        };

        let formatted = format!("{}", log);
        assert!(formatted.contains("HIT"));
        assert!(formatted.contains("2ms"));
    }

    #[test]
    fn test_response_log_ttl_extraction() {
        let ttls = vec![300, 600, 3600];
        let log = ResponseLog {
            domain_name: "test.com".to_string(),
            query_type: QueryType::MX,
            response_time_ms: 100,
            answer_count: 3,
            authority_count: 0,
            response_size: 200,
            was_cache_hit: false,
            ttl_values: ttls.clone(),
            result_code: ResultCode::NOERROR,
        };

        assert_eq!(log.ttl_values, ttls);
    }

    #[test]
    fn test_response_log_timing() {
        let log = ResponseLog {
            domain_name: "example.com".to_string(),
            query_type: QueryType::A,
            response_time_ms: 250,
            answer_count: 1,
            authority_count: 0,
            response_size: 64,
            was_cache_hit: false,
            ttl_values: vec![3600],
            result_code: ResultCode::NOERROR,
        };

        assert!(log.response_time_ms > 0);
        let formatted = format!("{}", log);
        assert!(formatted.contains("250ms"));
    }

    #[test]
    fn test_response_log_answer_count() {
        let log = ResponseLog {
            domain_name: "example.com".to_string(),
            query_type: QueryType::A,
            response_time_ms: 100,
            answer_count: 5,
            authority_count: 3,
            response_size: 512,
            was_cache_hit: false,
            ttl_values: vec![3600; 5],
            result_code: ResultCode::NOERROR,
        };

        assert_eq!(log.answer_count, 5);
        assert_eq!(log.authority_count, 3);
        let formatted = format!("{}", log);
        assert!(formatted.contains("Answers: 5"));
        assert!(formatted.contains("Authority: 3"));
    }
}
