use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path};

/// One cookie entry; domain/path could be added later if needed.
#[derive(Debug, Clone, Deserialize)]
pub struct CookieEntry {
    pub name: String,
    pub value: String,
}

/// The input JSON can be either a map or a list of cookie entries.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CookiesJson {
    Map(HashMap<String, String>),
    List(Vec<CookieEntry>),
}

/// Normalized cookie store (name -> value). We keep it simple for now.
/// If later we need domain/path scoping, we can extend this type.
#[derive(Debug, Clone)]
pub struct CookieStore {
    map: HashMap<String, String>,
}

impl CookieStore {
    /// Create a CookieStore from a serde_json::Value (already parsed).
    pub fn from_value(v: Value) -> anyhow::Result<Self> {
        // Try to deserialize into either a map or a list.
        let cj: CookiesJson = serde_json::from_value(v)?;
        let mut map = HashMap::new();

        match cj {
            CookiesJson::Map(m) => {
                // Direct mapping: { "name": "value", ... }
                map.extend(m);
            }
            CookiesJson::List(list) => {
                // Keep last occurrence on duplicates.
                for e in list {
                    map.insert(e.name, e.value);
                }
            }
        }

        Ok(Self { map })
    }

    /// Load cookies from a file path.
    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path)?;
        let v: Value = serde_json::from_str(&raw)?;
        Self::from_value(v)
    }

    /// Number of cookies.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Return a sorted list of cookie names (safe to log).
    pub fn cookie_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.map.keys().cloned().collect();
        names.sort();
        names
    }

    /// Render the `Cookie` header value, e.g.: "a=1; b=2".
    /// Deterministic order (by name) to help testing and reproducibility.
    pub fn to_header_value(&self) -> String {
        let mut pairs: Vec<_> = self.map.iter().collect();
        pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
        pairs
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::CookieStore;
    use serde_json::json;

    #[test]
    fn loads_from_map() {
        let v = json!({
            "sess": "abc",
            "OptanonConsent": "xyz"
        });
        let store = CookieStore::from_value(v).unwrap();
        assert_eq!(store.len(), 2);
        let names = store.cookie_names();
        assert_eq!(
            names,
            vec!["OptanonConsent".to_string(), "sess".to_string()]
        );
        let header = store.to_header_value();
        assert_eq!(header, "OptanonConsent=xyz; sess=abc");
    }

    #[test]
    fn loads_from_list() {
        let v = json!([
            { "name": "sess", "value": "abc" },
            { "name": "OptanonConsent", "value": "xyz", "domain": "learning.oreilly.com" }
        ]);
        let store = CookieStore::from_value(v).unwrap();
        assert_eq!(store.len(), 2);
        assert_eq!(store.cookie_names(), vec!["OptanonConsent", "sess"]);
        assert_eq!(store.to_header_value(), "OptanonConsent=xyz; sess=abc");
    }

    #[test]
    fn duplicate_names_keep_last() {
        let v = json!([
            { "name": "sess", "value": "OLD" },
            { "name": "sess", "value": "NEW" }
        ]);
        let store = CookieStore::from_value(v).unwrap();
        assert_eq!(store.len(), 1);
        assert_eq!(store.to_header_value(), "sess=NEW");
    }

    #[test]
    fn invalid_json_fails() {
        let v = serde_json::Value::String("not-json-shape".to_string());
        let err = CookieStore::from_value(v).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.to_lowercase().contains("did not match any variant"));
    }
}
