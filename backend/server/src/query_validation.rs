use chrono::{DateTime, Utc, Duration as ChronoDuration};
use anyhow::{Result, anyhow};

/// Validate and normalize history query parameters
pub struct HistoryQueryValidator {
    max_range_days: u32,
    max_results: usize,
}

impl HistoryQueryValidator {
    pub fn new(max_range_days: u32, max_results: usize) -> Self {
        Self {
            max_range_days,
            max_results,
        }
    }

    /// Validate and normalize history query parameters
    pub fn validate(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<u64>)> {
        let now = Utc::now();
        let max_range = ChronoDuration::days(self.max_range_days as i64);

        // Validate and set default 'to' time
        let to = to.unwrap_or(now);
        
        // Validate and set default 'from' time (max_range_days ago if not specified)
        let from = from.unwrap_or_else(|| {
            let default_from = now - max_range;
            if to < default_from {
                to - ChronoDuration::days(1) // If to is in the past, use 1 day range
            } else {
                default_from
            }
        });

        // Validate time range
        if from > to {
            return Err(anyhow!("'from' time must be before 'to' time"));
        }

        let range = to - from;
        if range > max_range {
            return Err(anyhow!(
                "Query range exceeds maximum of {} days",
                self.max_range_days
            ));
        }

        // Validate limit
        let limit = limit.map(|l| {
            if l as usize > self.max_results {
                self.max_results as u64
            } else {
                l
            }
        }).or(Some(self.max_results as u64));

        Ok((Some(from), Some(to), limit))
    }
}

/// Pagination parameters
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PaginationParams {
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub page_size: Option<u64>,
}

impl PaginationParams {
    pub fn normalize(&self, default_page_size: u64, max_page_size: u64) -> (u64, u64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size
            .unwrap_or(default_page_size)
            .min(max_page_size)
            .max(1);
        (page, page_size)
    }

    pub fn offset(&self, default_page_size: u64, max_page_size: u64) -> u64 {
        let (page, page_size) = self.normalize(default_page_size, max_page_size);
        (page - 1) * page_size
    }
}

/// Paginated response
#[derive(serde::Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(serde::Serialize)]
pub struct PaginationInfo {
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: u64, page_size: u64, total: u64) -> Self {
        let total_pages = (total as f64 / page_size as f64).ceil() as u64;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            data,
            pagination: PaginationInfo {
                page,
                page_size,
                total,
                total_pages,
                has_next,
                has_prev,
            },
        }
    }
}

