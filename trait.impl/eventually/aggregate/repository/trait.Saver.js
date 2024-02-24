(function() {var implementors = {
"eventually":[],
"eventually_postgres":[["impl&lt;T, Serde, EvtSerde&gt; <a class=\"trait\" href=\"eventually/aggregate/repository/trait.Saver.html\" title=\"trait eventually::aggregate::repository::Saver\">Saver</a>&lt;T&gt; for <a class=\"struct\" href=\"eventually_postgres/aggregate/struct.Repository.html\" title=\"struct eventually_postgres::aggregate::Repository\">Repository</a>&lt;T, Serde, EvtSerde&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,\n    &lt;T as <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a>&gt;::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Id\" title=\"type eventually::aggregate::Aggregate::Id\">Id</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/alloc/string/trait.ToString.html\" title=\"trait alloc::string::ToString\">ToString</a>,\n    Serde: <a class=\"trait\" href=\"eventually/serde/trait.Serde.html\" title=\"trait eventually::serde::Serde\">Serde</a>&lt;T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,\n    EvtSerde: <a class=\"trait\" href=\"eventually/serde/trait.Serde.html\" title=\"trait eventually::serde::Serde\">Serde</a>&lt;T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Event\" title=\"type eventually::aggregate::Aggregate::Event\">Event</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,</div>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()