//! Typed write counters carried in `ResultSummary.counters`.

use std::collections::HashMap;

/// Write counters of a statement, read from or written to the
/// `ResultSummary.counters` map.
///
/// The map is keyed by the `snake_case` names in [`Counters::KEYS`]. A key that
/// is missing counts as 0, and a server only needs to send the non-zero ones.
/// Other entries of the map (such as timing figures) are not write counters
/// and are ignored here.
///
/// Client side, use [`ResultCursor::counters`](crate::client::ResultCursor::counters).
/// Server side, [`insert_into`](Self::insert_into) fills a summary's map:
///
/// ```
/// use std::collections::HashMap;
/// use gwp::types::Counters;
///
/// let mut written = Counters::default();
/// written.nodes_created = 2;
/// written.properties_set = 4;
///
/// let mut map = HashMap::new();
/// map.insert("execution_time_ms".to_owned(), 3);
/// written.insert_into(&mut map);
/// assert_eq!(map["nodes_created"], 2);
/// assert!(!map.contains_key("edges_created"));
///
/// let read = Counters::from_map(&map);
/// assert_eq!(read, written);
/// assert!(read.contains_updates());
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Counters {
    /// Nodes created.
    pub nodes_created: u64,
    /// Nodes deleted.
    pub nodes_deleted: u64,
    /// Edges created.
    pub edges_created: u64,
    /// Edges deleted.
    pub edges_deleted: u64,
    /// Property values set or removed.
    pub properties_set: u64,
    /// Labels added.
    pub labels_added: u64,
    /// Labels removed.
    pub labels_removed: u64,
}

impl Counters {
    /// Key of [`nodes_created`](Self::nodes_created).
    pub const NODES_CREATED: &'static str = "nodes_created";
    /// Key of [`nodes_deleted`](Self::nodes_deleted).
    pub const NODES_DELETED: &'static str = "nodes_deleted";
    /// Key of [`edges_created`](Self::edges_created).
    pub const EDGES_CREATED: &'static str = "edges_created";
    /// Key of [`edges_deleted`](Self::edges_deleted).
    pub const EDGES_DELETED: &'static str = "edges_deleted";
    /// Key of [`properties_set`](Self::properties_set).
    pub const PROPERTIES_SET: &'static str = "properties_set";
    /// Key of [`labels_added`](Self::labels_added).
    pub const LABELS_ADDED: &'static str = "labels_added";
    /// Key of [`labels_removed`](Self::labels_removed).
    pub const LABELS_REMOVED: &'static str = "labels_removed";

    /// All write counter keys, in field order.
    pub const KEYS: [&'static str; 7] = [
        Self::NODES_CREATED,
        Self::NODES_DELETED,
        Self::EDGES_CREATED,
        Self::EDGES_DELETED,
        Self::PROPERTIES_SET,
        Self::LABELS_ADDED,
        Self::LABELS_REMOVED,
    ];

    /// Read the write counters from a summary's counter map.
    ///
    /// Missing keys read as 0. The wire carries `int64`; a negative value
    /// (which no well-behaved server sends) also reads as 0.
    #[must_use]
    pub fn from_map(map: &HashMap<String, i64>) -> Self {
        let get = |key: &str| {
            map.get(key)
                .map_or(0, |&value| u64::try_from(value).unwrap_or(0))
        };
        Self {
            nodes_created: get(Self::NODES_CREATED),
            nodes_deleted: get(Self::NODES_DELETED),
            edges_created: get(Self::EDGES_CREATED),
            edges_deleted: get(Self::EDGES_DELETED),
            properties_set: get(Self::PROPERTIES_SET),
            labels_added: get(Self::LABELS_ADDED),
            labels_removed: get(Self::LABELS_REMOVED),
        }
    }

    /// The counters as `(key, value)` pairs, in field order.
    #[must_use]
    pub fn entries(&self) -> [(&'static str, u64); 7] {
        [
            (Self::NODES_CREATED, self.nodes_created),
            (Self::NODES_DELETED, self.nodes_deleted),
            (Self::EDGES_CREATED, self.edges_created),
            (Self::EDGES_DELETED, self.edges_deleted),
            (Self::PROPERTIES_SET, self.properties_set),
            (Self::LABELS_ADDED, self.labels_added),
            (Self::LABELS_REMOVED, self.labels_removed),
        ]
    }

    /// Write the non-zero counters into a summary's counter map.
    ///
    /// Values above `i64::MAX` are written as `i64::MAX`. Zero counters are
    /// not written, and other entries of the map are left alone.
    pub fn insert_into(&self, map: &mut HashMap<String, i64>) {
        for (key, value) in self.entries() {
            if value > 0 {
                map.insert(key.to_owned(), i64::try_from(value).unwrap_or(i64::MAX));
            }
        }
    }

    /// Returns `true` if any counter is non-zero, that is, if the statement
    /// changed the graph.
    #[must_use]
    pub fn contains_updates(&self) -> bool {
        self.entries().iter().any(|&(_, value)| value > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_keys_read_as_zero() {
        let counters = Counters::from_map(&HashMap::new());
        assert_eq!(counters, Counters::default());
        assert!(!counters.contains_updates());
    }

    #[test]
    fn reads_every_key_and_ignores_others() {
        let map: HashMap<String, i64> = Counters::KEYS
            .iter()
            .zip(1..)
            .map(|(key, value)| ((*key).to_owned(), value))
            .chain([
                ("execution_time_ms".to_owned(), 99),
                ("rows_scanned".to_owned(), 1000),
            ])
            .collect();

        let counters = Counters::from_map(&map);
        assert_eq!(counters.nodes_created, 1);
        assert_eq!(counters.nodes_deleted, 2);
        assert_eq!(counters.edges_created, 3);
        assert_eq!(counters.edges_deleted, 4);
        assert_eq!(counters.properties_set, 5);
        assert_eq!(counters.labels_added, 6);
        assert_eq!(counters.labels_removed, 7);
        assert!(counters.contains_updates());
    }

    #[test]
    fn negative_values_read_as_zero() {
        let map = HashMap::from([
            (Counters::NODES_CREATED.to_owned(), -5),
            (Counters::EDGES_CREATED.to_owned(), i64::MIN),
        ]);
        assert_eq!(Counters::from_map(&map), Counters::default());
    }

    #[test]
    fn insert_saturates_and_skips_zero() {
        let counters = Counters {
            labels_removed: u64::MAX,
            edges_deleted: 1,
            ..Counters::default()
        };

        let mut map = HashMap::from([("rows_scanned".to_owned(), 7)]);
        counters.insert_into(&mut map);

        assert_eq!(map.len(), 3);
        assert_eq!(map[Counters::LABELS_REMOVED], i64::MAX);
        assert_eq!(map[Counters::EDGES_DELETED], 1);
        assert_eq!(map["rows_scanned"], 7);

        let back = Counters::from_map(&map);
        assert_eq!(back.labels_removed, i64::MAX.unsigned_abs());
        assert_eq!(back.edges_deleted, 1);
    }

    #[test]
    fn single_counter_counts_as_update() {
        for (index, key) in Counters::KEYS.iter().enumerate() {
            let map = HashMap::from([((*key).to_owned(), 1)]);
            let counters = Counters::from_map(&map);
            assert!(counters.contains_updates(), "{key}");
            assert_eq!(counters.entries()[index], (*key, 1));
        }
    }
}
