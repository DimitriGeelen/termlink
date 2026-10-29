//! The address book of a provisioned sidecar, mapping each agent of its project to a provisioned instance id.
//! The address book of a provisioned sidecar (PH-2 row P7; SI-20 pin, SI-21
//! fixed default, R-62 exact instance; operator rulings P7-1 to P7-4).
//!
//! In PH-2 an agent may send only to agents of its own project, named in an
//! operator-written book that maps each agent to its provisioned instance id.
//! `main` is the operator-pinned main agent. A send may leave `to_instance`
//! out and get the book's id; a different id is refused, never redirected.
//! Without a book (the PH-1 estate) sends are unchanged: `to_instance` required.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The alias that resolves to the pinned main agent.
pub const MAIN: &str = "main";

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Book {
    /// Agent name -> its provisioned instance id.
    pub peers: BTreeMap<String, String>,
    /// The operator-pinned main agent (SI-20); may be this agent itself.
    pub main: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Refused {
    pub code: &'static str,
    pub message: String,
}

/// The same rule the API applies to every name field (`api.rs` valid_name): a
/// book entry the API would refuse is refused at start, not at the first send.
fn plain(n: &str) -> bool {
    !n.is_empty()
        && n != "."
        && n != ".."
        && n.len() <= 200
        && n.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':'))
}

fn refused(code: &'static str, message: String) -> Refused {
    Refused { code, message }
}

impl Book {
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    /// Check the book against the agent that owns it; a bad book is a refused start.
    pub fn validate(&self, me: &str) -> Result<(), String> {
        if self.peers.contains_key(me) {
            return Err(format!("address book lists this agent ({me}) as a peer"));
        }
        if self.peers.contains_key(MAIN) {
            return Err(format!("address book has a peer named {MAIN:?}, which is the main alias"));
        }
        if let Some(m) = &self.main
            && m != me
            && !self.peers.contains_key(m)
        {
            return Err(format!("pinned main {m:?} is neither this agent nor in the address book"));
        }
        for (a, i) in &self.peers {
            if !plain(a) || !plain(i) {
                return Err(format!("address book entry {a:?} -> {i:?} is not plain names (letters, digits, . _ - :)"));
            }
        }
        Ok(())
    }

    /// Resolve a send's recipient to `(agent, instance)`.
    pub fn resolve(&self, me: &str, to: &str, to_instance: Option<&str>) -> Result<(String, String), Refused> {
        let agent = if to == MAIN {
            match &self.main {
                None => return Err(refused("no-main", "no main agent is pinned for this project".into())),
                Some(m) if m == me => return Err(refused("is-self", format!("the pinned main is this agent ({me})"))),
                Some(m) => m.clone(),
            }
        } else {
            to.to_owned()
        };
        let Some(book_inst) = self.peers.get(&agent) else {
            return Err(refused(
                "not-in-address-book",
                format!("{agent} is not an agent of this project's address book (PH-2 sends stay inside the project)"),
            ));
        };
        match to_instance {
            Some(i) if i != book_inst => Err(refused(
                "instance-mismatch",
                format!("{agent} is provisioned as instance {book_inst}, not {i}; mail is never redirected"),
            )),
            _ => Ok((agent, book_inst.clone())),
        }
    }
}

/// T-3619 (OD-18 ruling C): who holds main, for the `status` call. `instance` is the book's provisioned id
/// for a peer main; for main = this agent the book has no entry for itself, so it is null. No book (the
/// PH-1 estate) is reported, never an error.
pub fn main_view(book: Option<&Book>, me: &str) -> serde_json::Value {
    match book {
        None => serde_json::json!({"book": false, "pinned": null, "instance": null, "is_self": false}),
        Some(b) => {
            let pinned = b.main.as_deref();
            let is_self = pinned == Some(me);
            let instance = pinned.filter(|_| !is_self).and_then(|m| b.peers.get(m)).cloned();
            serde_json::json!({"book": true, "pinned": pinned, "instance": instance, "is_self": is_self})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_view_names_the_pin_its_instance_and_whether_it_is_this_agent() {
        let b = book();
        let v = main_view(Some(&b), "claude-termlink-alt");
        assert_eq!(v["pinned"], "claude-termlink");
        assert_eq!(v["is_self"], false);
        assert!(v["instance"].is_null(), "claude-termlink is not a peer in this book, so no instance");
        let peer_main = Book { main: Some("claude-termlink-alt".into()), ..book() };
        let v = main_view(Some(&peer_main), "claude-termlink");
        assert_eq!(v["instance"], "claude-termlink-alt.w1-20261013");
        assert_eq!(v["is_self"], false);
        let v = main_view(Some(&b), "claude-termlink");
        assert_eq!(v["is_self"], true);
        assert!(v["instance"].is_null());
        let none = Book { main: None, ..book() };
        assert!(main_view(Some(&none), "claude-termlink")["pinned"].is_null());
        let v = main_view(None, "x");
        assert_eq!(v["book"], false);
        assert!(v["pinned"].is_null());
    }

    fn book() -> Book {
        Book {
            peers: BTreeMap::from([("claude-termlink-alt".to_string(), "claude-termlink-alt.w1-20261013".to_string())]),
            main: Some("claude-termlink".into()),
        }
    }

    #[test]
    fn resolves_a_peer_and_fills_its_instance() {
        let b = book();
        assert_eq!(
            b.resolve("claude-termlink", "claude-termlink-alt", None).unwrap(),
            ("claude-termlink-alt".to_string(), "claude-termlink-alt.w1-20261013".to_string())
        );
        assert!(b.resolve("claude-termlink", "claude-termlink-alt", Some("claude-termlink-alt.w1-20261013")).is_ok());
    }

    #[test]
    fn refuses_outside_the_book_and_a_different_instance() {
        let b = book();
        assert_eq!(b.resolve("claude-termlink", "penelope", None).unwrap_err().code, "not-in-address-book");
        assert_eq!(b.resolve("claude-termlink", "claude-termlink-alt", Some("old")).unwrap_err().code, "instance-mismatch");
    }

    #[test]
    fn main_resolves_to_the_pin_and_never_to_self() {
        let b = book();
        assert_eq!(b.resolve("claude-termlink", MAIN, None).unwrap_err().code, "is-self");
        let alt = Book {
            peers: BTreeMap::from([("claude-termlink".to_string(), "claude-termlink.w1-20261013".to_string())]),
            main: Some("claude-termlink".into()),
        };
        assert_eq!(alt.resolve("claude-termlink-alt", MAIN, None).unwrap().0, "claude-termlink");
        let none = Book { main: None, ..alt };
        assert_eq!(none.resolve("claude-termlink-alt", MAIN, None).unwrap_err().code, "no-main");
    }

    #[test]
    fn validate_refuses_self_as_peer_a_main_named_peer_and_an_unknown_main() {
        let mut b = book();
        assert!(b.validate("claude-termlink").is_ok());
        b.peers.insert("claude-termlink".into(), "x".into());
        assert!(b.validate("claude-termlink").is_err());
        let mut c = book();
        c.peers.insert(MAIN.into(), "x".into());
        assert!(c.validate("claude-termlink").is_err());
        let d = Book { main: Some("stranger".into()), ..book() };
        assert!(d.validate("claude-termlink").is_err());
        let mut e = book();
        e.peers.insert("x".into(), "x@w1".into());
        assert!(e.validate("claude-termlink").unwrap_err().contains("plain names"));
    }
}
