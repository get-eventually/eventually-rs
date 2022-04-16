(function() {var implementors = {};
implementors["bank_accounting"] = [{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a>&lt;R&gt;","synthetic":true,"types":["bank_accounting::application::Service"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/application/struct.OpenBankAccount.html\" title=\"struct bank_accounting::application::OpenBankAccount\">OpenBankAccount</a>","synthetic":true,"types":["bank_accounting::application::OpenBankAccount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/application/struct.DepositInBankAccount.html\" title=\"struct bank_accounting::application::DepositInBankAccount\">DepositInBankAccount</a>","synthetic":true,"types":["bank_accounting::application::DepositInBankAccount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/application/struct.SendTransferToBankAccount.html\" title=\"struct bank_accounting::application::SendTransferToBankAccount\">SendTransferToBankAccount</a>","synthetic":true,"types":["bank_accounting::application::SendTransferToBankAccount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/domain/struct.Transaction.html\" title=\"struct bank_accounting::domain::Transaction\">Transaction</a>","synthetic":true,"types":["bank_accounting::domain::Transaction"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountEvent.html\" title=\"enum bank_accounting::domain::BankAccountEvent\">BankAccountEvent</a>","synthetic":true,"types":["bank_accounting::domain::BankAccountEvent"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountError.html\" title=\"enum bank_accounting::domain::BankAccountError\">BankAccountError</a>","synthetic":true,"types":["bank_accounting::domain::BankAccountError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>","synthetic":true,"types":["bank_accounting::domain::BankAccount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccountRoot.html\" title=\"struct bank_accounting::domain::BankAccountRoot\">BankAccountRoot</a>","synthetic":true,"types":["bank_accounting::domain::BankAccountRoot"]},{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/grpc/struct.BankAccountingApi.html\" title=\"struct bank_accounting::grpc::BankAccountingApi\">BankAccountingApi</a>&lt;R&gt;","synthetic":true,"types":["bank_accounting::grpc::BankAccountingApi"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/bank_accounting_server/struct.BankAccountingServer.html\" title=\"struct bank_accounting::proto::bank_accounting_server::BankAccountingServer\">BankAccountingServer</a>&lt;T&gt;","synthetic":true,"types":["bank_accounting::proto::bank_accounting_server::BankAccountingServer"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.OpenBankAccountRequest.html\" title=\"struct bank_accounting::proto::OpenBankAccountRequest\">OpenBankAccountRequest</a>","synthetic":true,"types":["bank_accounting::proto::OpenBankAccountRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.OpenBankAccountResponse.html\" title=\"struct bank_accounting::proto::OpenBankAccountResponse\">OpenBankAccountResponse</a>","synthetic":true,"types":["bank_accounting::proto::OpenBankAccountResponse"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.DepositInBankAccountRequest.html\" title=\"struct bank_accounting::proto::DepositInBankAccountRequest\">DepositInBankAccountRequest</a>","synthetic":true,"types":["bank_accounting::proto::DepositInBankAccountRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.DepositInBankAccountResponse.html\" title=\"struct bank_accounting::proto::DepositInBankAccountResponse\">DepositInBankAccountResponse</a>","synthetic":true,"types":["bank_accounting::proto::DepositInBankAccountResponse"]}];
implementors["eventually"] = [{"text":"impl&lt;E, SE, AE&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually/aggregate/enum.EventSourcedError.html\" title=\"enum eventually::aggregate::EventSourcedError\">EventSourcedError</a>&lt;E, SE, AE&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;AE: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;SE: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::aggregate::repository::EventSourcedError"]},{"text":"impl&lt;T, R, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/aggregate/struct.EventSourcedRepository.html\" title=\"struct eventually::aggregate::EventSourcedRepository\">EventSourced</a>&lt;T, R, S&gt;","synthetic":true,"types":["eventually::aggregate::repository::EventSourced"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/aggregate/struct.Context.html\" title=\"struct eventually::aggregate::Context\">Context</a>&lt;T&gt;","synthetic":true,"types":["eventually::aggregate::root::Context"]},{"text":"impl&lt;Id, Evt&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/event/struct.Persisted.html\" title=\"struct eventually::event::Persisted\">Persisted</a>&lt;Id, Evt&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Evt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::event::Persisted"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually/event/enum.VersionSelect.html\" title=\"enum eventually::event::VersionSelect\">VersionSelect</a>","synthetic":true,"types":["eventually::event::VersionSelect"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually/event/enum.StreamVersionExpected.html\" title=\"enum eventually::event::StreamVersionExpected\">StreamVersionExpected</a>","synthetic":true,"types":["eventually::event::StreamVersionExpected"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/message/struct.Envelope.html\" title=\"struct eventually::message::Envelope\">Envelope</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::message::Envelope"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/test/command_handler/struct.Scenario.html\" title=\"struct eventually::test::command_handler::Scenario\">Scenario</a>","synthetic":true,"types":["eventually::test::command_handler::Scenario"]},{"text":"impl&lt;Id, Evt&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/test/store/struct.InMemory.html\" title=\"struct eventually::test::store::InMemory\">InMemory</a>&lt;Id, Evt&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Evt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::test::store::InMemory"]},{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/test/store/struct.Tracking.html\" title=\"struct eventually::test::store::Tracking\">Tracking</a>&lt;S&gt;","synthetic":true,"types":["eventually::test::store::Tracking"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/version/struct.ConflictError.html\" title=\"struct eventually::version::ConflictError\">ConflictError</a>","synthetic":true,"types":["eventually::version::ConflictError"]}];
implementors["eventually_postgres"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually_postgres/store/enum.Error.html\" title=\"enum eventually_postgres::store::Error\">Error</a>","synthetic":true,"types":["eventually_postgres::store::Error"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_postgres/store/struct.EventStoreBuilder.html\" title=\"struct eventually_postgres::store::EventStoreBuilder\">EventStoreBuilder</a>","synthetic":true,"types":["eventually_postgres::store::EventStoreBuilder"]},{"text":"impl&lt;Tls&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_postgres/store/struct.EventStoreBuilderMigrated.html\" title=\"struct eventually_postgres::store::EventStoreBuilderMigrated\">EventStoreBuilderMigrated</a>&lt;Tls&gt;","synthetic":true,"types":["eventually_postgres::store::EventStoreBuilderMigrated"]},{"text":"impl&lt;Id, Event, Tls&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_postgres/store/struct.EventStore.html\" title=\"struct eventually_postgres::store::EventStore\">EventStore</a>&lt;Id, Event, Tls&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Event: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually_postgres::store::EventStore"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually_postgres/subscriber/enum.SubscriberError.html\" title=\"enum eventually_postgres::subscriber::SubscriberError\">SubscriberError</a>","synthetic":true,"types":["eventually_postgres::subscriber::SubscriberError"]},{"text":"impl&lt;Id, Event&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_postgres/subscriber/struct.EventSubscriber.html\" title=\"struct eventually_postgres::subscriber::EventSubscriber\">EventSubscriber</a>&lt;Id, Event&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Event: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,&nbsp;</span>","synthetic":true,"types":["eventually_postgres::subscriber::EventSubscriber"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually_postgres/subscription/enum.Error.html\" title=\"enum eventually_postgres::subscription::Error\">Error</a>","synthetic":true,"types":["eventually_postgres::subscription::Error"]},{"text":"impl&lt;SourceId, Event, Tls&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_postgres/subscription/struct.PersistentBuilder.html\" title=\"struct eventually_postgres::subscription::PersistentBuilder\">PersistentBuilder</a>&lt;SourceId, Event, Tls&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Event: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;SourceId: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually_postgres::subscription::PersistentBuilder"]},{"text":"impl&lt;SourceId, Event, Tls&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_postgres/subscription/struct.Persistent.html\" title=\"struct eventually_postgres::subscription::Persistent\">Persistent</a>&lt;SourceId, Event, Tls&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Event: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;SourceId: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually_postgres::subscription::Persistent"]}];
implementors["eventually_redis"] = [{"text":"impl&lt;Id, Event&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_redis/struct.EventStore.html\" title=\"struct eventually_redis::EventStore\">EventStore</a>&lt;Id, Event&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Event: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually_redis::store::EventStore"]},{"text":"impl&lt;Id, Event&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_redis/struct.EventSubscriber.html\" title=\"struct eventually_redis::EventSubscriber\">EventSubscriber</a>&lt;Id, Event&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Event: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually_redis::subscriber::EventSubscriber"]},{"text":"impl&lt;Id, Event&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_redis/struct.PersistentSubscription.html\" title=\"struct eventually_redis::PersistentSubscription\">PersistentSubscription</a>&lt;Id, Event&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Event: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually_redis::subscription::PersistentSubscription"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_redis/struct.Builder.html\" title=\"struct eventually_redis::Builder\">Builder</a>","synthetic":true,"types":["eventually_redis::Builder"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_redis/struct.BuilderWithSourceName.html\" title=\"struct eventually_redis::BuilderWithSourceName\">BuilderWithSourceName</a>","synthetic":true,"types":["eventually_redis::BuilderWithSourceName"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()