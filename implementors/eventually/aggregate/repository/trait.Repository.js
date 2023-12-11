(function() {var implementors = {
"eventually":[],
"eventually_postgres":[["impl&lt;T, OutT, OutEvt, TSerde, EvtSerde&gt; Repository&lt;T&gt; for <a class=\"struct\" href=\"eventually_postgres/aggregate/struct.Repository.html\" title=\"struct eventually_postgres::aggregate::Repository\">Repository</a>&lt;T, OutT, OutEvt, TSerde, EvtSerde&gt;<span class=\"where fmt-newline\">where\n    T: Aggregate + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;OutT&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,\n    &lt;T as Aggregate&gt;::Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/alloc/string/trait.ToString.html\" title=\"trait alloc::string::ToString\">ToString</a>,\n    &lt;T as <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;OutT&gt;&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/1.74.1/core/convert/trait.TryFrom.html#associatedtype.Error\" title=\"type core::convert::TryFrom::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,\n    OutT: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,\n    OutEvt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T::Event&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,\n    TSerde: Serde&lt;OutT&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,\n    &lt;TSerde as Serde&lt;OutT&gt;&gt;::Error: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,\n    EvtSerde: Serde&lt;OutEvt&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()