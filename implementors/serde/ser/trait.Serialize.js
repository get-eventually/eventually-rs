(function() {var implementors = {};
implementors["eventually"] = [{"text":"impl&lt;Id, Evt&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.136/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a> for <a class=\"struct\" href=\"eventually/event/struct.Persisted.html\" title=\"struct eventually::event::Persisted\">Persisted</a>&lt;Id, Evt&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Evt: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Id: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.136/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Evt: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.136/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a>,&nbsp;</span>","synthetic":false,"types":["eventually::event::Persisted"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.136/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a> for <a class=\"struct\" href=\"eventually/message/struct.Envelope.html\" title=\"struct eventually::message::Envelope\">Envelope</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.136/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a>,&nbsp;</span>","synthetic":false,"types":["eventually::message::Envelope"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()