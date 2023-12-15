(function() {var implementors = {
"bank_accounting":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountError.html\" title=\"enum bank_accounting::domain::BankAccountError\">BankAccountError</a>"]],
"eventually":[["impl&lt;R, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually/aggregate/repository/event_sourced/enum.GetError.html\" title=\"enum eventually::aggregate::repository::event_sourced::GetError\">GetError</a>&lt;R, S&gt;<span class=\"where fmt-newline\">where\n    R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + 'static,\n    S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + 'static,\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,</span>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually/aggregate/repository/event_sourced/enum.SaveError.html\" title=\"enum eventually::aggregate::repository::event_sourced::SaveError\">SaveError</a>&lt;T&gt;<span class=\"where fmt-newline\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + 'static,\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,</span>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"eventually/version/struct.ConflictError.html\" title=\"struct eventually::version::ConflictError\">ConflictError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"eventually/aggregate/repository/any/struct.AnyError.html\" title=\"struct eventually::aggregate::repository::any::AnyError\">AnyError</a>"],["impl&lt;I&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually/aggregate/repository/enum.GetError.html\" title=\"enum eventually::aggregate::repository::GetError\">GetError</a>&lt;I&gt;<span class=\"where fmt-newline\">where\n    I: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + 'static,\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,</span>"]],
"eventually_postgres":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/event/enum.AppendError.html\" title=\"enum eventually_postgres::event::AppendError\">AppendError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/aggregate/enum.SaveError.html\" title=\"enum eventually_postgres::aggregate::SaveError\">SaveError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/event/enum.StreamError.html\" title=\"enum eventually_postgres::event::StreamError\">StreamError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/aggregate/enum.GetError.html\" title=\"enum eventually_postgres::aggregate::GetError\">GetError</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()