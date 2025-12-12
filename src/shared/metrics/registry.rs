/*
use lazy_static::lazy_static;

lazy_static! {
    // HTTP Metrics
    pub static ref HTTP_REQUESTS_TOTAL: &'static str = "http_requests_total";
    pub static ref HTTP_REQUEST_DURATION_SECONDS: &'static str = "http_request_duration_seconds";
    pub static ref HTTP_REQUEST_BODY_SIZE_BYTES: &'static str = "http_request_body_size_bytes";
    pub static ref HTTP_RESPONSE_BODY_SIZE_BYTES: &'static str = "http_response_body_size_bytes";

    // Database Metrics
    pub static ref DATABASE_QUERIES_TOTAL: &'static str = "database_queries_total";
    pub static ref DATABASE_QUERY_DURATION_SECONDS: &'static str = "database_query_duration_seconds";
    pub static ref DATABASE_CONNECTIONS_TOTAL: &'static str = "database_connections_total";
    pub static ref DATABASE_CONNECTION_ERRORS_TOTAL: &'static str = "database_connection_errors_total";

    // Redis Metrics
    pub static ref REDIS_OPERATIONS_TOTAL: &'static str = "redis_operations_total";
    pub static ref REDIS_OPERATION_DURATION_SECONDS: &'static str = "redis_operation_duration_seconds";
    pub static ref REDIS_CACHE_HITS_TOTAL: &'static str = "redis_cache_hits_total";
    pub static ref REDIS_CACHE_MISSES_TOTAL: &'static str = "redis_cache_misses_total";

    // Business Metrics
    pub static ref BUSINESS_EVENTS_TOTAL: &'static str = "business_events_total";
    pub static ref BUSINESS_OPERATION_DURATION_SECONDS: &'static str = "business_operation_duration_seconds";

    // Application Metrics
    pub static ref APPLICATION_INFO: &'static str = "application_info";
    pub static ref APPLICATION_START_TIME_SECONDS: &'static str = "application_start_time_seconds";
    pub static ref APPLICATION_UPTIME_SECONDS: &'static str = "application_uptime_seconds";
    pub static ref APPLICATION_MEMORY_USAGE_BYTES: &'static str = "application_memory_usage_bytes";
}

pub fn register_custom_metrics() {
    use metrics::{describe_counter, describe_histogram, describe_gauge};

    // HTTP Metrics
    describe_counter!(
        *HTTP_REQUESTS_TOTAL,
        "Total number of HTTP requests made to the API"
    );
    describe_histogram!(
        *HTTP_REQUEST_DURATION_SECONDS,
        "HTTP request duration in seconds"
    );
    describe_histogram!(
        *HTTP_REQUEST_BODY_SIZE_BYTES,
        "HTTP request body size in bytes"
    );
    describe_histogram!(
        *HTTP_RESPONSE_BODY_SIZE_BYTES,
        "HTTP response body size in bytes"
    );

    // Database Metrics
    describe_counter!(
        *DATABASE_QUERIES_TOTAL,
        "Total number of database queries executed"
    );
    describe_histogram!(
        *DATABASE_QUERY_DURATION_SECONDS,
        "Database query duration in seconds"
    );
    describe_gauge!(
        *DATABASE_CONNECTIONS_TOTAL,
        "Current number of database connections"
    );
    describe_counter!(
        *DATABASE_CONNECTION_ERRORS_TOTAL,
        "Total number of database connection errors"
    );

    // Redis Metrics
    describe_counter!(
        *REDIS_OPERATIONS_TOTAL,
        "Total number of Redis operations"
    );
    describe_histogram!(
        *REDIS_OPERATION_DURATION_SECONDS,
        "Redis operation duration in seconds"
    );
    describe_counter!(
        *REDIS_CACHE_HITS_TOTAL,
        "Total number of Redis cache hits"
    );
    describe_counter!(
        *REDIS_CACHE_MISSES_TOTAL,
        "Total number of Redis cache misses"
    );

    // Business Metrics
    describe_counter!(
        *BUSINESS_EVENTS_TOTAL,
        "Total number of business events"
    );
    describe_histogram!(
        *BUSINESS_OPERATION_DURATION_SECONDS,
        "Business operation duration in seconds"
    );

    // Application Metrics
    describe_gauge!(
        *APPLICATION_INFO,
        "Application information"
    );
    describe_gauge!(
        *APPLICATION_START_TIME_SECONDS,
        "Application start time in seconds since epoch"
    );
    describe_gauge!(
        *APPLICATION_UPTIME_SECONDS,
        "Application uptime in seconds"
    );
    describe_gauge!(
        *APPLICATION_MEMORY_USAGE_BYTES,
        "Application memory usage in bytes"
    );
}
*/