mod model;
mod tui;

pub use model::Item;

pub struct Multiselect {
    prompt: String,
    items: Vec<Item>,
}

impl Multiselect {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            items: Vec::new(),
        }
    }

    pub fn items(mut self, items: Vec<Item>) -> Self {
        self.items = items;
        self
    }

    pub fn run(self) -> Result<Option<Vec<String>>, String> {
        let tree = model::Tree::build(self.items)?;
        tui::run(&self.prompt, tree)
    }
}
