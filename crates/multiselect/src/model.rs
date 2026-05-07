use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub label: String,
    pub parent: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderRow {
    pub item_idx: usize,
    pub depth: usize,
    pub has_children: bool,
}

#[derive(Debug)]
pub(crate) struct Tree {
    pub items: Vec<Item>,
    roots: Vec<usize>,
    children: Vec<Vec<usize>>,
}

impl Tree {
    pub fn build(items: Vec<Item>) -> Result<Self, String> {
        let mut id_to_index: HashMap<String, usize> = HashMap::with_capacity(items.len());
        for (i, it) in items.iter().enumerate() {
            if id_to_index.insert(it.id.clone(), i).is_some() {
                return Err(format!("duplicate id: {}", it.id));
            }
        }
        for it in items.iter() {
            if let Some(p) = &it.parent {
                if !id_to_index.contains_key(p) {
                    return Err(format!("item {:?} has unknown parent: {:?}", it.id, p));
                }
            }
        }

        let mut children: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
        let mut roots: Vec<usize> = Vec::new();
        for (i, it) in items.iter().enumerate() {
            match &it.parent {
                Some(p) => {
                    let pi = id_to_index[p];
                    children[pi].push(i);
                }
                None => roots.push(i),
            }
        }

        Ok(Self { items, roots, children })
    }

    pub fn render_order(&self) -> Vec<RenderRow> {
        let mut out = Vec::with_capacity(self.items.len());
        for &r in &self.roots {
            self.dfs(r, 0, &mut out);
        }
        out
    }

    fn dfs(&self, i: usize, depth: usize, out: &mut Vec<RenderRow>) {
        out.push(RenderRow {
            item_idx: i,
            depth,
            has_children: !self.children[i].is_empty(),
        });
        for &c in &self.children[i] {
            self.dfs(c, depth + 1, out);
        }
    }

    pub fn toggle(&mut self, idx: usize) {
        let new_state = !self.items[idx].selected;
        self.items[idx].selected = new_state;
        let mut stack: Vec<usize> = self.children[idx].clone();
        while let Some(d) = stack.pop() {
            self.items[d].selected = new_state;
            stack.extend(self.children[d].iter().copied());
        }
    }

    pub fn selected_leaves(&self) -> Vec<String> {
        self.render_order()
            .into_iter()
            .filter(|r| !r.has_children && self.items[r.item_idx].selected)
            .map(|r| self.items[r.item_idx].id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, parent: Option<&str>, selected: bool) -> Item {
        Item {
            id: id.to_string(),
            label: id.to_string(),
            parent: parent.map(|s| s.to_string()),
            selected,
        }
    }

    fn ids(rows: &[RenderRow], items: &[Item]) -> Vec<String> {
        rows.iter().map(|r| items[r.item_idx].id.clone()).collect()
    }

    #[test]
    fn render_order_groups_children_under_parents() {
        // input order interleaves parents and a stray top-level leaf,
        // children for "a" appear before parent "b" in input but should
        // be grouped under "a" on render.
        let t = Tree::build(vec![
            item("a", None, false),
            item("b", None, false),
            item("a1", Some("a"), false),
            item("a2", Some("a"), false),
            item("b1", Some("b"), false),
            item("c", None, false),
        ])
        .unwrap();
        let rows = t.render_order();
        assert_eq!(ids(&rows, &t.items), vec!["a", "a1", "a2", "b", "b1", "c"]);
    }

    #[test]
    fn render_order_preserves_sibling_order() {
        let t = Tree::build(vec![
            item("p", None, false),
            item("c2", Some("p"), false),
            item("c1", Some("p"), false), // caller's order: c2 before c1
            item("c3", Some("p"), false),
        ])
        .unwrap();
        let rows = t.render_order();
        assert_eq!(ids(&rows, &t.items), vec!["p", "c2", "c1", "c3"]);
    }

    #[test]
    fn depth_reflects_nesting() {
        let t = Tree::build(vec![
            item("a", None, false),
            item("b", Some("a"), false),
            item("c", Some("b"), false),
        ])
        .unwrap();
        let rows = t.render_order();
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].depth, 2);
    }

    #[test]
    fn has_children_flag_is_correct() {
        let t = Tree::build(vec![
            item("p", None, false),
            item("c", Some("p"), false),
            item("solo", None, false),
        ])
        .unwrap();
        let rows = t.render_order();
        assert!(rows[0].has_children); // p
        assert!(!rows[1].has_children); // c
        assert!(!rows[2].has_children); // solo
    }

    #[test]
    fn toggle_propagates_to_all_descendants() {
        let mut t = Tree::build(vec![
            item("a", None, false),
            item("b", Some("a"), false),
            item("c", Some("b"), false),
            item("d", Some("a"), false),
        ])
        .unwrap();
        t.toggle(0); // toggle a → all descendants become true
        assert!(t.items[0].selected);
        assert!(t.items[1].selected);
        assert!(t.items[2].selected);
        assert!(t.items[3].selected);
        t.toggle(0); // toggle a again → all descendants become false
        assert!(!t.items[0].selected);
        assert!(!t.items[1].selected);
        assert!(!t.items[2].selected);
        assert!(!t.items[3].selected);
    }

    #[test]
    fn toggle_leaf_does_not_affect_parent() {
        let mut t = Tree::build(vec![
            item("p", None, false),
            item("c", Some("p"), false),
        ])
        .unwrap();
        t.toggle(1); // toggle c
        assert!(t.items[1].selected);
        assert!(!t.items[0].selected);
    }

    #[test]
    fn selected_leaves_returns_only_leaves_in_render_order() {
        let mut t = Tree::build(vec![
            item("a", None, true),  // parent, marked selected — should be excluded
            item("a2", Some("a"), false),
            item("a1", Some("a"), true),
            item("b", None, true), // top-level leaf
            item("c", None, false),
            item("c1", Some("c"), true),
        ])
        .unwrap();
        // Note: a is a parent so it's excluded even though selected=true.
        // a1 selected, a2 not selected. b leaf selected. c1 leaf selected.
        let _ = &mut t; // keep mutability future-proof
        assert_eq!(t.selected_leaves(), vec!["a1", "b", "c1"]);
    }

    #[test]
    fn duplicate_id_errors() {
        let err = Tree::build(vec![
            item("a", None, false),
            item("a", None, false),
        ])
        .unwrap_err();
        assert!(err.contains("duplicate"));
    }

    #[test]
    fn dangling_parent_errors() {
        let err = Tree::build(vec![item("a", Some("nope"), false)]).unwrap_err();
        assert!(err.contains("unknown parent"));
    }

    #[test]
    fn empty_input_is_allowed() {
        let t = Tree::build(vec![]).unwrap();
        assert!(t.render_order().is_empty());
        assert!(t.selected_leaves().is_empty());
    }
}
