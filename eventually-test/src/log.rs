use chrono::Utc;

use futures::future::BoxFuture;

use tide::{Middleware, Next, Request, Result};

/// Log all incoming requests and responses.
#[derive(Debug, Clone)]
pub struct LogMiddleware {
    _priv: (),
}

impl LogMiddleware {
    /// Create a new instance of `LogMiddleware`.
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Log a request and a response.
    async fn log<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> Result {
        let path = ctx.uri().path().to_owned();
        let method = ctx.method().to_string();

        log::trace!("IN => {} {}", method, path);

        let start = Utc::now();
        let result = next.run(ctx).await;
        let end = Utc::now()
            .signed_duration_since(start)
            .to_std()
            .map(Duration::from)?;

        result
            .map(|result| {
                let status = result.status();
                log::info!("{} {} {} {}", method, path, status, end);
                result
            })
            .map_err(|err| {
                let msg = err.to_string();
                log::error!("{} {} {} {}", msg, method, path, end);
                err
            })
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for LogMiddleware {
    fn handle<'a>(&'a self, ctx: Request<State>, next: Next<'a, State>) -> BoxFuture<'a, Result> {
        Box::pin(async move { self.log(ctx, next).await })
    }
}

#[derive(Debug)]
struct Duration(std::time::Duration);

use std::fmt::{Display, Formatter, Result as FmtResult};

impl From<std::time::Duration> for Duration {
    fn from(t: std::time::Duration) -> Self {
        Self(t)
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let time = self.0;

        if time.as_secs() > 0 {
            Self::fmt_seconds(time.as_secs(), f)
        } else if time.as_millis() > 0 {
            write!(f, "{}ms", time.as_millis())
        } else if time.as_micros() > 0 {
            write!(f, "{}us", time.as_micros())
        } else {
            write!(f, "{}ns", time.as_nanos())
        }
    }
}

impl Duration {
    fn fmt_seconds(mut seconds: u64, f: &mut Formatter) -> FmtResult {
        let mut minutes = seconds / 60;
        let mut hours = minutes / 60;
        let days = hours / 24;

        if days > 0 {
            write!(f, "{}d", days)?;
            hours -= days * 24;
            minutes -= days * 24 * 60;
            seconds -= days * 24 * 60 * 60;
        }

        if hours > 0 {
            write!(f, "{}h", hours)?;
            minutes -= hours * 60;
            seconds -= hours * 60 * 60;
        }

        if minutes > 0 {
            write!(f, "{}m", minutes)?;
            seconds -= minutes * 60;
        }

        write!(f, "{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn duration_display_multiple_days_correctly() {
        use chrono::Duration;

        let start =
            Duration::days(1) + Duration::hours(2) + Duration::minutes(10) + Duration::seconds(30);

        let duration = start.to_std().map(super::Duration::from).unwrap();

        assert_eq!("1d2h10m30s", format!("{}", duration));
    }
}
