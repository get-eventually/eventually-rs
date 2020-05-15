use chrono::Utc;

use futures::future::BoxFuture;

use tide::{Next, Request, Result};

/// Log all incoming requests and responses.
#[derive(Debug, Clone)]
pub struct Middleware {
    _priv: (),
}

impl Middleware {
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
        let end = Utc::now().signed_duration_since(start).to_std()?;

        result
            .map(|result| {
                let status = result.status();
                log::info!("{} {} {} {:?}", method, path, status, end);
                result
            })
            .map_err(|err| {
                let msg = err.to_string();
                log::error!("{} {} {} {:?}", msg, method, path, end);
                err
            })
    }
}

impl<State: Send + Sync + 'static> tide::Middleware<State> for Middleware {
    fn handle<'a>(&'a self, ctx: Request<State>, next: Next<'a, State>) -> BoxFuture<'a, Result> {
        Box::pin(async move { self.log(ctx, next).await })
    }
}
