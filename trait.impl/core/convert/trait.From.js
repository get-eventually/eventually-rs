(function() {var implementors = {
"bank_accounting":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountEvent.html\" title=\"enum bank_accounting::domain::BankAccountEvent\">BankAccountEvent</a>&gt; for <a class=\"struct\" href=\"bank_accounting/proto/struct.Event.html\" title=\"struct bank_accounting::proto::Event\">Event</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a>&gt; for <a class=\"struct\" href=\"bank_accounting/grpc/struct.BankAccountingApi.html\" title=\"struct bank_accounting::grpc::BankAccountingApi\">BankAccountingApi</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccountRoot.html\" title=\"struct bank_accounting::domain::BankAccountRoot\">BankAccountRoot</a>&gt; for <a class=\"struct\" href=\"eventually/aggregate/struct.Root.html\" title=\"struct eventually::aggregate::Root\">Root</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"bank_accounting/proto/struct.Event.html\" title=\"struct bank_accounting::proto::Event\">Event</a>&gt; for <a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountEvent.html\" title=\"enum bank_accounting::domain::BankAccountEvent\">BankAccountEvent</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.Transaction.html\" title=\"struct bank_accounting::domain::Transaction\">Transaction</a>&gt; for <a class=\"struct\" href=\"bank_accounting/proto/struct.Transaction.html\" title=\"struct bank_accounting::proto::Transaction\">Transaction</a>"],["impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;R&gt; for <a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a><div class=\"where\">where\n    R: <a class=\"trait\" href=\"eventually/aggregate/repository/trait.Repository.html\" title=\"trait eventually::aggregate::repository::Repository\">Repository</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>&gt; + 'static,</div>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"eventually/aggregate/struct.Root.html\" title=\"struct eventually::aggregate::Root\">Root</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>&gt;&gt; for <a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccountRoot.html\" title=\"struct bank_accounting::domain::BankAccountRoot\">BankAccountRoot</a>"]],
"eventually":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T&gt; for <a class=\"struct\" href=\"eventually/message/struct.Envelope.html\" title=\"struct eventually::message::Envelope\">Envelope</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a>,</div>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/anyhow/1.0.80/anyhow/struct.Error.html\" title=\"struct anyhow::Error\">Error</a>&gt; for <a class=\"enum\" href=\"eventually/event/store/enum.AppendError.html\" title=\"enum eventually::event::store::AppendError\">AppendError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/anyhow/1.0.80/anyhow/struct.Error.html\" title=\"struct anyhow::Error\">Error</a>&gt; for <a class=\"enum\" href=\"eventually/aggregate/repository/enum.SaveError.html\" title=\"enum eventually::aggregate::repository::SaveError\">SaveError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"eventually/version/struct.ConflictError.html\" title=\"struct eventually::version::ConflictError\">ConflictError</a>&gt; for <a class=\"enum\" href=\"eventually/aggregate/repository/enum.SaveError.html\" title=\"enum eventually::aggregate::repository::SaveError\">SaveError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/anyhow/1.0.80/anyhow/struct.Error.html\" title=\"struct anyhow::Error\">Error</a>&gt; for <a class=\"enum\" href=\"eventually/aggregate/repository/enum.GetError.html\" title=\"enum eventually::aggregate::repository::GetError\">GetError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"eventually/version/struct.ConflictError.html\" title=\"struct eventually::version::ConflictError\">ConflictError</a>&gt; for <a class=\"enum\" href=\"eventually/event/store/enum.AppendError.html\" title=\"enum eventually::event::store::AppendError\">AppendError</a>"],["impl&lt;T, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;S&gt; for <a class=\"struct\" href=\"eventually/aggregate/repository/struct.EventSourced.html\" title=\"struct eventually::aggregate::repository::EventSourced\">EventSourced</a>&lt;T, S&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a>,\n    S: <a class=\"trait\" href=\"eventually/event/store/trait.Store.html\" title=\"trait eventually::event::store::Store\">Store</a>&lt;T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Id\" title=\"type eventually::aggregate::Aggregate::Id\">Id</a>, T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Event\" title=\"type eventually::aggregate::Aggregate::Event\">Event</a>&gt;,</div>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()