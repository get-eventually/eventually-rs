(function() {var implementors = {
"bank_accounting":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountError.html\" title=\"enum bank_accounting::domain::BankAccountError\">BankAccountError</a>"]],
"eventually":[["impl&lt;I&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually/aggregate/repository/enum.GetError.html\" title=\"enum eventually::aggregate::repository::GetError\">GetError</a>&lt;I&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;I: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,</span>"],["impl&lt;E, SE, AE&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually/aggregate/repository/enum.EventSourcedError.html\" title=\"enum eventually::aggregate::repository::EventSourcedError\">EventSourcedError</a>&lt;E, SE, AE&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;SE: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;AE: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,</span>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"eventually/version/struct.ConflictError.html\" title=\"struct eventually::version::ConflictError\">ConflictError</a>"]],
"eventually_postgres":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/aggregate/enum.GetError.html\" title=\"enum eventually_postgres::aggregate::GetError\">GetError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/aggregate/enum.SaveError.html\" title=\"enum eventually_postgres::aggregate::SaveError\">SaveError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/event/enum.StreamError.html\" title=\"enum eventually_postgres::event::StreamError\">StreamError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/event/enum.AppendError.html\" title=\"enum eventually_postgres::event::AppendError\">AppendError</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()