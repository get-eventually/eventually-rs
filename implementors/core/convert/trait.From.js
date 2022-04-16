(function() {var implementors = {};
implementors["bank_accounting"] = [{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;R&gt; for <a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a>&lt;R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"eventually/aggregate/repository/trait.Repository.html\" title=\"trait eventually::aggregate::repository::Repository\">Repository</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>, <a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccountRoot.html\" title=\"struct bank_accounting::domain::BankAccountRoot\">BankAccountRoot</a>&gt;,&nbsp;</span>","synthetic":false,"types":["bank_accounting::application::Service"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"eventually/aggregate/root/struct.Context.html\" title=\"struct eventually::aggregate::root::Context\">Context</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>&gt;&gt; for <a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccountRoot.html\" title=\"struct bank_accounting::domain::BankAccountRoot\">BankAccountRoot</a>","synthetic":false,"types":["bank_accounting::domain::BankAccountRoot"]},{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a>&lt;R&gt;&gt; for <a class=\"struct\" href=\"bank_accounting/grpc/struct.BankAccountingApi.html\" title=\"struct bank_accounting::grpc::BankAccountingApi\">BankAccountingApi</a>&lt;R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"eventually/aggregate/repository/trait.Repository.html\" title=\"trait eventually::aggregate::repository::Repository\">Repository</a>&lt;<a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>, <a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccountRoot.html\" title=\"struct bank_accounting::domain::BankAccountRoot\">BankAccountRoot</a>&gt;,&nbsp;</span>","synthetic":false,"types":["bank_accounting::grpc::BankAccountingApi"]}];
implementors["eventually"] = [{"text":"impl&lt;T, R, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;S&gt; for <a class=\"struct\" href=\"eventually/aggregate/struct.EventSourcedRepository.html\" title=\"struct eventually::aggregate::EventSourcedRepository\">EventSourced</a>&lt;T, R, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"eventually/aggregate/trait.Root.html\" title=\"trait eventually::aggregate::Root\">Root</a>&lt;T&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"eventually/event/trait.Store.html\" title=\"trait eventually::event::Store\">Store</a>&lt;StreamId = T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Id\" title=\"type eventually::aggregate::Aggregate::Id\">Id</a>, Event = T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Event\" title=\"type eventually::aggregate::Aggregate::Event\">Event</a>&gt;,&nbsp;</span>","synthetic":false,"types":["eventually::aggregate::repository::EventSourced"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T&gt; for <a class=\"struct\" href=\"eventually/message/struct.Envelope.html\" title=\"struct eventually::message::Envelope\">Envelope</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a>,&nbsp;</span>","synthetic":false,"types":["eventually::message::Envelope"]}];
implementors["eventually_postgres"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/tokio-postgres/0.7/tokio_postgres/error/struct.Error.html\" title=\"struct tokio_postgres::error::Error\">Error</a>&gt; for <a class=\"enum\" href=\"eventually_postgres/store/enum.Error.html\" title=\"enum eventually_postgres::store::Error\">Error</a>","synthetic":false,"types":["eventually_postgres::store::Error"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;RunError&lt;<a class=\"struct\" href=\"https://docs.rs/tokio-postgres/0.7/tokio_postgres/error/struct.Error.html\" title=\"struct tokio_postgres::error::Error\">Error</a>&gt;&gt; for <a class=\"enum\" href=\"eventually_postgres/store/enum.Error.html\" title=\"enum eventually_postgres::store::Error\">Error</a>","synthetic":false,"types":["eventually_postgres::store::Error"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()