(function() {var implementors = {};
implementors["bank_accounting"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountError.html\" title=\"enum bank_accounting::domain::BankAccountError\">BankAccountError</a>","synthetic":false,"types":["bank_accounting::domain::BankAccountError"]}];
implementors["eventually"] = [{"text":"impl&lt;I&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually/aggregate/enum.RepositoryGetError.html\" title=\"enum eventually::aggregate::RepositoryGetError\">GetError</a>&lt;I&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;I: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,&nbsp;</span>","synthetic":false,"types":["eventually::aggregate::repository::GetError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"struct\" href=\"eventually/version/struct.ConflictError.html\" title=\"struct eventually::version::ConflictError\">ConflictError</a>","synthetic":false,"types":["eventually::version::ConflictError"]}];
implementors["eventually_postgres"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/aggregate/enum.GetError.html\" title=\"enum eventually_postgres::aggregate::GetError\">GetError</a>","synthetic":false,"types":["eventually_postgres::aggregate::GetError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/aggregate/enum.SaveError.html\" title=\"enum eventually_postgres::aggregate::SaveError\">SaveError</a>","synthetic":false,"types":["eventually_postgres::aggregate::SaveError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/event/enum.StreamError.html\" title=\"enum eventually_postgres::event::StreamError\">StreamError</a>","synthetic":false,"types":["eventually_postgres::event::StreamError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"enum\" href=\"eventually_postgres/event/enum.AppendError.html\" title=\"enum eventually_postgres::event::AppendError\">AppendError</a>","synthetic":false,"types":["eventually_postgres::event::AppendError"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()