(function() {var implementors = {};
implementors["bank_accounting"] = [{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/application/struct.Service.html\" title=\"struct bank_accounting::application::Service\">Service</a>&lt;R&gt;","synthetic":true,"types":["bank_accounting::application::Service"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/application/struct.OpenBankAccount.html\" title=\"struct bank_accounting::application::OpenBankAccount\">OpenBankAccount</a>","synthetic":true,"types":["bank_accounting::application::OpenBankAccount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/application/struct.DepositInBankAccount.html\" title=\"struct bank_accounting::application::DepositInBankAccount\">DepositInBankAccount</a>","synthetic":true,"types":["bank_accounting::application::DepositInBankAccount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/application/struct.SendTransferToBankAccount.html\" title=\"struct bank_accounting::application::SendTransferToBankAccount\">SendTransferToBankAccount</a>","synthetic":true,"types":["bank_accounting::application::SendTransferToBankAccount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/domain/struct.Transaction.html\" title=\"struct bank_accounting::domain::Transaction\">Transaction</a>","synthetic":true,"types":["bank_accounting::domain::Transaction"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountEvent.html\" title=\"enum bank_accounting::domain::BankAccountEvent\">BankAccountEvent</a>","synthetic":true,"types":["bank_accounting::domain::BankAccountEvent"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"bank_accounting/domain/enum.BankAccountError.html\" title=\"enum bank_accounting::domain::BankAccountError\">BankAccountError</a>","synthetic":true,"types":["bank_accounting::domain::BankAccountError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccount.html\" title=\"struct bank_accounting::domain::BankAccount\">BankAccount</a>","synthetic":true,"types":["bank_accounting::domain::BankAccount"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/domain/struct.BankAccountRoot.html\" title=\"struct bank_accounting::domain::BankAccountRoot\">BankAccountRoot</a>","synthetic":true,"types":["bank_accounting::domain::BankAccountRoot"]},{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/grpc/struct.BankAccountingApi.html\" title=\"struct bank_accounting::grpc::BankAccountingApi\">BankAccountingApi</a>&lt;R&gt;","synthetic":true,"types":["bank_accounting::grpc::BankAccountingApi"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/bank_accounting_server/struct.BankAccountingServer.html\" title=\"struct bank_accounting::proto::bank_accounting_server::BankAccountingServer\">BankAccountingServer</a>&lt;T&gt;","synthetic":true,"types":["bank_accounting::proto::bank_accounting_server::BankAccountingServer"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/event/struct.WasOpened.html\" title=\"struct bank_accounting::proto::event::WasOpened\">WasOpened</a>","synthetic":true,"types":["bank_accounting::proto::event::WasOpened"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/event/struct.DepositWasRecorded.html\" title=\"struct bank_accounting::proto::event::DepositWasRecorded\">DepositWasRecorded</a>","synthetic":true,"types":["bank_accounting::proto::event::DepositWasRecorded"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/event/struct.TransferWasSent.html\" title=\"struct bank_accounting::proto::event::TransferWasSent\">TransferWasSent</a>","synthetic":true,"types":["bank_accounting::proto::event::TransferWasSent"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/event/struct.TransferWasReceived.html\" title=\"struct bank_accounting::proto::event::TransferWasReceived\">TransferWasReceived</a>","synthetic":true,"types":["bank_accounting::proto::event::TransferWasReceived"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/event/struct.TransferWasDeclined.html\" title=\"struct bank_accounting::proto::event::TransferWasDeclined\">TransferWasDeclined</a>","synthetic":true,"types":["bank_accounting::proto::event::TransferWasDeclined"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/event/struct.TransferWasConfirmed.html\" title=\"struct bank_accounting::proto::event::TransferWasConfirmed\">TransferWasConfirmed</a>","synthetic":true,"types":["bank_accounting::proto::event::TransferWasConfirmed"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/event/struct.WasClosed.html\" title=\"struct bank_accounting::proto::event::WasClosed\">WasClosed</a>","synthetic":true,"types":["bank_accounting::proto::event::WasClosed"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/event/struct.WasReopened.html\" title=\"struct bank_accounting::proto::event::WasReopened\">WasReopened</a>","synthetic":true,"types":["bank_accounting::proto::event::WasReopened"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"bank_accounting/proto/event/enum.Event.html\" title=\"enum bank_accounting::proto::event::Event\">Event</a>","synthetic":true,"types":["bank_accounting::proto::event::Event"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.OpenBankAccountRequest.html\" title=\"struct bank_accounting::proto::OpenBankAccountRequest\">OpenBankAccountRequest</a>","synthetic":true,"types":["bank_accounting::proto::OpenBankAccountRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.OpenBankAccountResponse.html\" title=\"struct bank_accounting::proto::OpenBankAccountResponse\">OpenBankAccountResponse</a>","synthetic":true,"types":["bank_accounting::proto::OpenBankAccountResponse"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.DepositInBankAccountRequest.html\" title=\"struct bank_accounting::proto::DepositInBankAccountRequest\">DepositInBankAccountRequest</a>","synthetic":true,"types":["bank_accounting::proto::DepositInBankAccountRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.DepositInBankAccountResponse.html\" title=\"struct bank_accounting::proto::DepositInBankAccountResponse\">DepositInBankAccountResponse</a>","synthetic":true,"types":["bank_accounting::proto::DepositInBankAccountResponse"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.Transaction.html\" title=\"struct bank_accounting::proto::Transaction\">Transaction</a>","synthetic":true,"types":["bank_accounting::proto::Transaction"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"bank_accounting/proto/struct.Event.html\" title=\"struct bank_accounting::proto::Event\">Event</a>","synthetic":true,"types":["bank_accounting::proto::Event"]}];
implementors["eventually"] = [{"text":"impl&lt;I&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually/aggregate/enum.RepositoryGetError.html\" title=\"enum eventually::aggregate::RepositoryGetError\">GetError</a>&lt;I&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;I: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::aggregate::repository::GetError"]},{"text":"impl&lt;T, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/aggregate/struct.EventSourcedRepository.html\" title=\"struct eventually::aggregate::EventSourcedRepository\">EventSourced</a>&lt;T, S&gt;","synthetic":true,"types":["eventually::aggregate::repository::EventSourced"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/aggregate/struct.Root.html\" title=\"struct eventually::aggregate::Root\">Root</a>&lt;T&gt;","synthetic":true,"types":["eventually::aggregate::Root"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/command/test/struct.Scenario.html\" title=\"struct eventually::command::test::Scenario\">Scenario</a>","synthetic":true,"types":["eventually::command::test::Scenario"]},{"text":"impl&lt;Id, Evt&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/event/store/struct.InMemory.html\" title=\"struct eventually::event::store::InMemory\">InMemory</a>&lt;Id, Evt&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Evt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::event::store::InMemory"]},{"text":"impl&lt;T, StreamId, Event&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/event/store/struct.Tracking.html\" title=\"struct eventually::event::store::Tracking\">Tracking</a>&lt;T, StreamId, Event&gt;","synthetic":true,"types":["eventually::event::store::Tracking"]},{"text":"impl&lt;Id, Evt&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/event/struct.Persisted.html\" title=\"struct eventually::event::Persisted\">Persisted</a>&lt;Id, Evt&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Evt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::event::Persisted"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually/event/enum.VersionSelect.html\" title=\"enum eventually::event::VersionSelect\">VersionSelect</a>","synthetic":true,"types":["eventually::event::VersionSelect"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually/event/enum.StreamVersionExpected.html\" title=\"enum eventually::event::StreamVersionExpected\">StreamVersionExpected</a>","synthetic":true,"types":["eventually::event::StreamVersionExpected"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/message/struct.Envelope.html\" title=\"struct eventually::message::Envelope\">Envelope</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::message::Envelope"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/serde/json/struct.Json.html\" title=\"struct eventually::serde::json::Json\">Json</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually::serde::json::Json"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/serde/prost/struct.MessageSerde.html\" title=\"struct eventually::serde::prost::MessageSerde\">MessageSerde</a>&lt;T&gt;","synthetic":true,"types":["eventually::serde::prost::MessageSerde"]},{"text":"impl&lt;T, Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/tracing/struct.InstrumentedAggregateRepository.html\" title=\"struct eventually::tracing::InstrumentedAggregateRepository\">InstrumentedAggregateRepository</a>&lt;T, Inner&gt;","synthetic":true,"types":["eventually::tracing::InstrumentedAggregateRepository"]},{"text":"impl&lt;T, StreamId, Event&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/tracing/struct.InstrumentedEventStore.html\" title=\"struct eventually::tracing::InstrumentedEventStore\">InstrumentedEventStore</a>&lt;T, StreamId, Event&gt;","synthetic":true,"types":["eventually::tracing::InstrumentedEventStore"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually/version/struct.ConflictError.html\" title=\"struct eventually::version::ConflictError\">ConflictError</a>","synthetic":true,"types":["eventually::version::ConflictError"]}];
implementors["eventually_postgres"] = [{"text":"impl&lt;T, OutT, OutEvt, TSerde, EvtSerde&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_postgres/aggregate/struct.Repository.html\" title=\"struct eventually_postgres::aggregate::Repository\">Repository</a>&lt;T, OutT, OutEvt, TSerde, EvtSerde&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;EvtSerde: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OutEvt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OutT: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;TSerde: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually_postgres::aggregate::Repository"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually_postgres/aggregate/enum.GetError.html\" title=\"enum eventually_postgres::aggregate::GetError\">GetError</a>","synthetic":true,"types":["eventually_postgres::aggregate::GetError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually_postgres/aggregate/enum.SaveError.html\" title=\"enum eventually_postgres::aggregate::SaveError\">SaveError</a>","synthetic":true,"types":["eventually_postgres::aggregate::SaveError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually_postgres/event/enum.StreamError.html\" title=\"enum eventually_postgres::event::StreamError\">StreamError</a>","synthetic":true,"types":["eventually_postgres::event::StreamError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"eventually_postgres/event/enum.AppendError.html\" title=\"enum eventually_postgres::event::AppendError\">AppendError</a>","synthetic":true,"types":["eventually_postgres::event::AppendError"]},{"text":"impl&lt;Id, Evt, OutEvt, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"eventually_postgres/event/struct.Store.html\" title=\"struct eventually_postgres::event::Store\">Store</a>&lt;Id, Evt, OutEvt, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Evt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OutEvt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["eventually_postgres::event::Store"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()