(function() {var implementors = {};
implementors["eventually"] = [];
implementors["eventually_postgres"] = [{"text":"impl&lt;T, OutT, OutEvt, TSerde, EvtSerde&gt; <a class=\"trait\" href=\"eventually/aggregate/repository/trait.Getter.html\" title=\"trait eventually::aggregate::repository::Getter\">Getter</a>&lt;T&gt; for <a class=\"struct\" href=\"eventually_postgres/aggregate/struct.Repository.html\" title=\"struct eventually_postgres::aggregate::Repository\">Repository</a>&lt;T, OutT, OutEvt, TSerde, EvtSerde&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;OutT&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as <a class=\"trait\" href=\"eventually/aggregate/trait.Aggregate.html\" title=\"trait eventually::aggregate::Aggregate\">Aggregate</a>&gt;::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Id\" title=\"type eventually::aggregate::Aggregate::Id\">Id</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/alloc/string/trait.ToString.html\" title=\"trait alloc::string::ToString\">ToString</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;OutT&gt;&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/1.64.0/core/convert/trait.TryFrom.html#associatedtype.Error\" title=\"type core::convert::TryFrom::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;OutT: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OutEvt: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T::<a class=\"associatedtype\" href=\"eventually/aggregate/trait.Aggregate.html#associatedtype.Event\" title=\"type eventually::aggregate::Aggregate::Event\">Event</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;TSerde: <a class=\"trait\" href=\"eventually/serde/trait.Serde.html\" title=\"trait eventually::serde::Serde\">Serde</a>&lt;OutT&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;TSerde as <a class=\"trait\" href=\"eventually/serde/trait.Deserializer.html\" title=\"trait eventually::serde::Deserializer\">Deserializer</a>&lt;OutT&gt;&gt;::<a class=\"associatedtype\" href=\"eventually/serde/trait.Deserializer.html#associatedtype.Error\" title=\"type eventually::serde::Deserializer::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;EvtSerde: <a class=\"trait\" href=\"eventually/serde/trait.Serializer.html\" title=\"trait eventually::serde::Serializer\">Serializer</a>&lt;OutEvt&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.64.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":false,"types":["eventually_postgres::aggregate::Repository"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()