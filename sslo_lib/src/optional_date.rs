use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct OptionalDateTime(Option<DateTime<Utc>>);

impl OptionalDateTime {
    pub fn new(date: Option<DateTime<Utc>>) -> Self { Self(date) }

    pub fn raw(&self) -> &Option<DateTime<Utc>> { &self.0 }

    /// date and time
    pub fn html_label_full(&self) -> String {
        match self.0 {
            None => "Never".to_string(),
            Some(date) => {
                format!("<div class=\"OptionalDateTime\"><div class=\"OptionalDate\">{}</div> <div class=\"OptionalTime\">{}</div></div>",
                        date.format("%Y-%m-%d"),
                        date.format("%H:%M:%SZ"),
                )
            }
        }
    }

    /// only date
    pub fn html_label_date(&self) -> String {
        match self.0 {
            None => "Never".to_string(),
            Some(date) => {
                format!("<div class=\"OptionalDateTime\"><div class=\"OptionalDate\">{}</div></div>",
                        date.format("%Y-%m-%d"),
                )
            }
        }
    }

    /// only time
    pub fn html_label_time(&self) -> String {
        match self.0 {
            None => "Never".to_string(),
            Some(date) => {
                format!("<div class=\"OptionalDateTime\"><div class=\"OptionalTime\">{}</div></div>",
                        date.format("%H:%M:%SZ"),
                )
            }
        }
    }
}
