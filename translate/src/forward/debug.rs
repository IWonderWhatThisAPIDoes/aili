//! Pretty-printing [`Renderer`]s.

use super::Renderer;
use aili_model::{state::NodeId, vis::VisTree};

impl<T, V> std::fmt::Debug for Renderer<'_, T, V>
where
    T: NodeId + Ord,
    V: VisTree,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            self.pretty_print_fmt(f)
        } else {
            self.debug_fmt(f)
        }
    }
}

impl<T, V> Renderer<'_, T, V>
where
    T: NodeId,
    V: VisTree,
{
    /// Formats the state of a renderer in a way fit for inline debug printing.
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ root: {:?}, mapping: {{ ", self.current_root)?;
        for (key, rendering) in &self.current_mappping {
            write!(f, "[{key:?}]: {:?}, ", rendering.properties)?;
        }
        write!(f, "}} }}")?;
        Ok(())
    }
}

impl<T, V> Renderer<'_, T, V>
where
    T: NodeId + Ord,
    V: VisTree,
{
    /// Formats the szaze of a renderer as a table for ease of reading.
    fn pretty_print_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Since we are pretty-printing, sort the table by keys
        // so we have them in a readable order (this is why we require Ord to be implemented)
        let mut sorted_refs = self.current_mappping.iter().collect::<Vec<_>>();
        sorted_refs
            .sort_by_key(|(key, _)| (&key.node_id, key.edge_label.is_some(), &key.extra_label));
        // Format all entity IDs and property keys and values and cache them
        // Not very performant, but this is debug pretty-printing, so it is worth it
        let property_maps = sorted_refs
            .into_iter()
            .map(|(key, rendering)| {
                let key_printout = format!("{key:?}");
                let mut mapping_printouts = Vec::new();
                // Put together printouts for all properties
                if let Some(display) = &rendering.properties.display {
                    mapping_printouts.push(("display".to_owned(), format!("{display:?}")));
                }
                if let Some(parent) = &rendering.properties.parent {
                    mapping_printouts.push(("parent".to_owned(), format!("{parent:?}")));
                }
                if let Some(target) = &rendering.properties.target {
                    mapping_printouts.push(("target".to_owned(), format!("{target:?}")));
                }
                for (name, value) in &rendering.properties.attributes {
                    mapping_printouts.push((format!("{name:?}"), format!("{value:?}")));
                }
                for (fragment, attributes) in &rendering.properties.fragment_attributes {
                    for (name, value) in attributes {
                        mapping_printouts
                            .push((format!("{fragment:?}/{name:?}"), format!("{value:?}")));
                    }
                }
                (key_printout, mapping_printouts)
            })
            .collect::<Vec<_>>();
        // Prepare the root property as well
        // This is a property of the renderer as a whole, rather than
        // a particular entity
        let root_key = "root";
        let root_value = if let Some(root) = &self.current_root {
            format!("{root:?}")
        } else {
            "[none]".to_owned()
        };
        // Widths of display columns are given by the longest
        let width_1 = property_maps
            .iter()
            .map(|(x, _)| x.len())
            .max()
            .unwrap_or_default()
            .min(MAX_COLUMN_WIDTH);
        let width_2 = property_maps
            .iter()
            .flat_map(|(_, x)| x)
            .map(|(x, _)| x.len())
            .max()
            .unwrap_or_default()
            .max(root_key.len())
            .min(MAX_COLUMN_WIDTH);
        // Print the first row (the one where root is set)
        writeln!(f, "{:_<width_1$}_| {root_key:<width_2$} : {root_value}", "")?;
        // Print the rest of the table
        for (entity, properties) in property_maps {
            for (i, (key, value)) in properties.iter().enumerate() {
                if i == 0 {
                    // First row gets the entity identifier as well
                    if properties.len() == 1 {
                        // If it's also the last, write a separator in there
                        // so it's visually separated
                        write!(f, "{entity:_>width_1$}_")?;
                    } else {
                        write!(f, "{entity:>width_1$} ")?;
                    }
                } else if i == properties.len() - 1 {
                    // Last row (unless it is also the first)
                    // gets a separator for ease of reading
                    write!(f, "{:_<width_1$}_", "")?;
                } else {
                    write!(f, "{:<width_1$} ", "")?;
                }
                writeln!(f, "| {key:<width_2$} : {value}")?;
            }
        }
        Ok(())
    }
}

/// Maximum width to which a column can be padded
/// If someone brings in extremely long property/entity/value names,
/// this keeps the table reasonably narrow.
const MAX_COLUMN_WIDTH: usize = 150;
