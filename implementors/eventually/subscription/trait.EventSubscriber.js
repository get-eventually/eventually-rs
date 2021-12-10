(function() {var implementors = {};
implementors["eventually_postgres"] = [{"text":"impl&lt;Id, Event&gt; <a class=\"trait\" href=\"eventually/subscription/trait.EventSubscriber.html\" title=\"trait eventually::subscription::EventSubscriber\">EventSubscriber</a> for <a class=\"struct\" href=\"eventually_postgres/subscriber/struct.EventSubscriber.html\" title=\"struct eventually_postgres::subscriber::EventSubscriber\">EventSubscriber</a>&lt;Id, Event&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;Event: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + 'static,&nbsp;</span>","synthetic":false,"types":["eventually_postgres::subscriber::EventSubscriber"]}];
implementors["eventually_redis"] = [{"text":"impl&lt;Id, Event&gt; <a class=\"trait\" href=\"eventually/subscription/trait.EventSubscriber.html\" title=\"trait eventually::subscription::EventSubscriber\">EventSubscriber</a> for <a class=\"struct\" href=\"eventually_redis/struct.EventSubscriber.html\" title=\"struct eventually_redis::EventSubscriber\">EventSubscriber</a>&lt;Id, Event&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.57.0/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;Id as <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.57.0/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;&gt;::<a class=\"type\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.TryFrom.html#associatedtype.Error\" title=\"type core::convert::TryFrom::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;for&lt;'de&gt; Event: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.131/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":false,"types":["eventually_redis::subscriber::EventSubscriber"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()