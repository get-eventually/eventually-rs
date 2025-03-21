(function() {
    var implementors = Object.fromEntries([["eventually",[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"enum\" href=\"eventually/event/enum.VersionSelect.html\" title=\"enum eventually::event::VersionSelect\">VersionSelect</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"enum\" href=\"eventually/version/enum.Check.html\" title=\"enum eventually::version::Check\">Check</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/version/struct.ConflictError.html\" title=\"struct eventually::version::ConflictError\">ConflictError</a>"],["impl&lt;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>, Evt&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/event/store/struct.InMemory.html\" title=\"struct eventually::event::store::InMemory\">InMemory</a>&lt;Id, Evt&gt;<div class=\"where\">where\n    Evt: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"],["impl&lt;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>, Evt&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/event/struct.Persisted.html\" title=\"struct eventually::event::Persisted\">Persisted</a>&lt;Id, Evt&gt;<div class=\"where\">where\n    Evt: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"],["impl&lt;In, Out, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/serde/struct.Convert.html\" title=\"struct eventually::serde::Convert\">Convert</a>&lt;In, Out, S&gt;<div class=\"where\">where\n    In: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    Out: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    S: <a class=\"trait\" href=\"eventually/serde/trait.Serde.html\" title=\"trait eventually::serde::Serde\">Serde</a>&lt;Out&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/aggregate/struct.Root.html\" title=\"struct eventually::aggregate::Root\">Root</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Event\" title=\"type eventually::aggregate::Aggregate::Event\">Event</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/aggregate/test/struct.Scenario.html\" title=\"struct eventually::aggregate::test::Scenario\">Scenario</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Id\" title=\"type eventually::aggregate::Aggregate::Id\">Id</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Event\" title=\"type eventually::aggregate::Aggregate::Event\">Event</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>,\n    T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Error\" title=\"type eventually::aggregate::Aggregate::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</div>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/message/struct.Envelope.html\" title=\"struct eventually::message::Envelope\">Envelope</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/serde/struct.Json.html\" title=\"struct eventually::serde::Json\">Json</a>&lt;T&gt;<div class=\"where\">where\n    for&lt;'d&gt; T: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'d&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"],["impl&lt;T, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/aggregate/repository/struct.EventSourced.html\" title=\"struct eventually::aggregate::repository::EventSourced\">EventSourced</a>&lt;T, S&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    S: <a class=\"trait\" href=\"eventually/event/store/trait.Store.html\" title=\"trait eventually::event::store::Store\">Store</a>&lt;T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Id\" title=\"type eventually::aggregate::Aggregate::Id\">Id</a>, T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Event\" title=\"type eventually::aggregate::Aggregate::Event\">Event</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"],["impl&lt;T, StreamId, Event&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually/event/store/struct.Tracking.html\" title=\"struct eventually::event::store::Tracking\">Tracking</a>&lt;T, StreamId, Event&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/event/store/trait.Store.html\" title=\"trait eventually::event::store::Store\">Store</a>&lt;StreamId, Event&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    StreamId: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    Event: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"]]],["eventually_postgres",[["impl&lt;Id, Evt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>, Serde&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually_postgres/event/struct.Store.html\" title=\"struct eventually_postgres::event::Store\">Store</a>&lt;Id, Evt, Serde&gt;<div class=\"where\">where\n    Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/alloc/string/trait.ToString.html\" title=\"trait alloc::string::ToString\">ToString</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    Serde: <a class=\"trait\" href=\"eventually/serde/trait.Serde.html\" title=\"trait eventually::serde::Serde\">Serde</a>&lt;Evt&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"],["impl&lt;T, Serde, EvtSerde&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"eventually_postgres/aggregate/struct.Repository.html\" title=\"struct eventually_postgres::aggregate::Repository\">Repository</a>&lt;T, Serde, EvtSerde&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    &lt;T as <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a>&gt;::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Id\" title=\"type eventually::aggregate::Aggregate::Id\">Id</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/alloc/string/trait.ToString.html\" title=\"trait alloc::string::ToString\">ToString</a>,\n    Serde: <a class=\"trait\" href=\"eventually/serde/trait.Serde.html\" title=\"trait eventually::serde::Serde\">Serde</a>&lt;T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    EvtSerde: <a class=\"trait\" href=\"eventually/serde/trait.Serde.html\" title=\"trait eventually::serde::Serde\">Serde</a>&lt;T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Event\" title=\"type eventually::aggregate::Aggregate::Event\">Event</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[11407,3054]}