(function() {var implementors = {};
implementors["bank_accounting"] = [{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"eventually/command/trait.Handler.html\" title=\"trait eventually::command::Handler\">Handler</a>&lt;<a class=\"struct\" href=\"bank_accounting/application/struct.OpenBankAccount.html\" title=\"struct bank_accounting::application::OpenBankAccount\">OpenBankAccount</a>&gt; for <a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a>&lt;R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"eventually/aggregate/repository/trait.Repository.html\" title=\"trait eventually::aggregate::repository::Repository\">Repository</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;R::<a class=\"associatedtype\" href=\"eventually/aggregate/repository/trait.Repository.html#associatedtype.Error\" title=\"type eventually::aggregate::repository::Repository::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">StdError</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,&nbsp;</span>","synthetic":false,"types":["bank_accounting::application::Service"]},{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"eventually/command/trait.Handler.html\" title=\"trait eventually::command::Handler\">Handler</a>&lt;<a class=\"struct\" href=\"bank_accounting/application/struct.DepositInBankAccount.html\" title=\"struct bank_accounting::application::DepositInBankAccount\">DepositInBankAccount</a>&gt; for <a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a>&lt;R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"eventually/aggregate/repository/trait.Repository.html\" title=\"trait eventually::aggregate::repository::Repository\">Repository</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;R::<a class=\"associatedtype\" href=\"eventually/aggregate/repository/trait.Repository.html#associatedtype.Error\" title=\"type eventually::aggregate::repository::Repository::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">StdError</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,&nbsp;</span>","synthetic":false,"types":["bank_accounting::application::Service"]},{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"eventually/command/trait.Handler.html\" title=\"trait eventually::command::Handler\">Handler</a>&lt;<a class=\"struct\" href=\"bank_accounting/application/struct.SendTransferToBankAccount.html\" title=\"struct bank_accounting::application::SendTransferToBankAccount\">SendTransferToBankAccount</a>&gt; for <a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a>&lt;R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"eventually/aggregate/repository/trait.Repository.html\" title=\"trait eventually::aggregate::repository::Repository\">Repository</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;R::<a class=\"associatedtype\" href=\"eventually/aggregate/repository/trait.Repository.html#associatedtype.Error\" title=\"type eventually::aggregate::repository::Repository::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">StdError</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,&nbsp;</span>","synthetic":false,"types":["bank_accounting::application::Service"]}];
implementors["eventually"] = [];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()