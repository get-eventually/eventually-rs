(function() {
    var implementors = Object.fromEntries([["eventually",[["impl&lt;'de, Id, Evt&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"eventually/event/struct.Persisted.html\" title=\"struct eventually::event::Persisted\">Persisted</a>&lt;Id, Evt&gt;<div class=\"where\">where\n    Evt: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a> + <a class=\"trait\" href=\"https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt;,\n    Id: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt;,</div>"],["impl&lt;'de, T&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"eventually/message/struct.Envelope.html\" title=\"struct eventually::message::Envelope\">Envelope</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"eventually/message/trait.Message.html\" title=\"trait eventually::message::Message\">Message</a> + <a class=\"trait\" href=\"https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt;,</div>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[1481]}