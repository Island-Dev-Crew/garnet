//! S41 — concurrency contract surface.
//!
//! Garnet's concurrency model is **actors**, not async/await (`async` is reserved
//! for a future edition — S32). Each actor is an OS thread with a **bounded**
//! mpsc mailbox (the default capacity closes the unbounded-mailbox DoS class;
//! override per actor). Messages are *protocols*: a protocol that returns a value
//! is an **`ask`** (request-reply, Result-returning); one that returns nothing is
//! a **`tell`** (fire-and-forget). This extracts that contract per actor — the
//! canonical concurrency surface documented in
//! `C_Language_Specification/GARNET_CONCURRENCY_CONTRACT.md`.

use garnet_parser::ast::{ActorItem, Item, Module};

/// One actor message protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolSig {
    pub name: String,
    pub arity: usize,
    /// `true` if the protocol returns a value — an `ask` (request-reply,
    /// Result-returning); `false` is a `tell` (fire-and-forget).
    pub reply: bool,
}

/// The concurrency contract of a single actor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorContract {
    pub name: String,
    pub protocols: Vec<ProtocolSig>,
    pub handlers: usize,
}

/// Extract the per-actor concurrency contract (message protocols + handler
/// count), sorted by actor name and protocol name (deterministic).
pub fn concurrency_surface(module: &Module) -> Vec<ActorContract> {
    let mut out: Vec<ActorContract> = Vec::new();
    for item in &module.items {
        let Item::Actor(a) = item else { continue };
        let mut protocols = Vec::new();
        let mut handlers = 0usize;
        for ai in &a.items {
            match ai {
                ActorItem::Protocol(p) => protocols.push(ProtocolSig {
                    name: p.name.clone(),
                    arity: p.params.len(),
                    reply: p.return_ty.is_some(),
                }),
                ActorItem::Handler(_) => handlers += 1,
                ActorItem::Memory(_) | ActorItem::Let(_) => {}
            }
        }
        protocols.sort_by(|x, y| x.name.cmp(&y.name));
        out.push(ActorContract {
            name: a.name.clone(),
            protocols,
            handlers,
        });
    }
    out.sort_by(|x, y| x.name.cmp(&y.name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_parser::parse_source;

    fn surface(src: &str) -> Vec<ActorContract> {
        concurrency_surface(&parse_source(src).expect("parses"))
    }

    #[test]
    fn classifies_ask_vs_tell_protocols() {
        let s = surface(
            "actor Counter {\n  protocol incr()\n  protocol get() -> Int\n  on incr() { 1 }\n  on get() { 0 }\n}\n",
        );
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].name, "Counter");
        assert_eq!(s[0].handlers, 2);
        // protocols sorted by name: get (ask), incr (tell)
        assert_eq!(s[0].protocols[0].name, "get");
        assert!(s[0].protocols[0].reply, "get() -> Int is an ask");
        assert_eq!(s[0].protocols[1].name, "incr");
        assert!(!s[0].protocols[1].reply, "incr() (no return) is a tell");
    }

    #[test]
    fn actors_sorted_and_non_actor_modules_empty() {
        assert!(surface("@caps()\ndef main() { 1 }\n").is_empty());
        let s = surface(
            "actor Zebra { protocol z() -> Int\n on z() { 1 } }\nactor Alpha { protocol a() -> Int\n on a() { 1 } }\n",
        );
        assert_eq!(s[0].name, "Alpha");
        assert_eq!(s[1].name, "Zebra");
    }
}
