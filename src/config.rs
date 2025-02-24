use esp_metadata::Chip;

use crate::template::{GeneratorOption, GeneratorOptionItem};

pub struct ActiveConfiguration<'c> {
    /// The chip that is configured for
    pub chip: Chip,
    /// The names of the selected options
    pub selected: Vec<String>,
    /// All available options
    pub options: &'c [GeneratorOptionItem],
}

impl ActiveConfiguration<'_> {
    pub fn is_selected(&self, option: &str) -> bool {
        self.selected_index(option).is_some()
    }

    pub fn selected_index(&self, option: &str) -> Option<usize> {
        self.selected.iter().position(|s| s == option)
    }

    pub fn select(&mut self, option: String) {
        self.selected.push(option);
    }

    pub fn is_active(&self, item: &GeneratorOptionItem) -> bool {
        match item {
            GeneratorOptionItem::Category(options) => {
                for sub in options.options.iter() {
                    if self.is_active(sub) {
                        return true;
                    }
                }
                false
            }
            GeneratorOptionItem::Option(option) => self.requirements_met(option),
        }
    }

    pub fn requirements_met(&self, option: &GeneratorOption) -> bool {
        if !option.chips.is_empty() && !option.chips.contains(&self.chip) {
            return false;
        }

        // Are this option's requirements met?
        for requirement in option.requires.iter() {
            let (key, expected) = if let Some(requirement) = requirement.strip_prefix('!') {
                (requirement, false)
            } else {
                (requirement.as_str(), true)
            };

            if self.is_selected(key) != expected {
                return false;
            }
        }

        // Does any of the enabled options have a requirement against this one?
        for selected in self.selected.iter() {
            let Some(selected_option) = find_option(selected, self.options) else {
                ratatui::restore();
                panic!("selected option not found: {selected}");
            };

            for requirement in selected_option.requires.iter() {
                if let Some(requirement) = requirement.strip_prefix('!') {
                    if requirement == option.name {
                        return false;
                    }
                }
            }
        }

        true
    }

    // An option can only be disabled if it's not required by any other selected option.
    pub fn can_be_disabled(&self, option: &str) -> bool {
        for selected in self.selected.iter() {
            let Some(selected_option) = find_option(selected, self.options) else {
                ratatui::restore();
                panic!("selected option not found: {selected}");
            };
            if selected_option.requires.iter().any(|o| o == option) {
                return false;
            }
        }
        true
    }
}

pub fn find_option<'c>(
    option: &str,
    options: &'c [GeneratorOptionItem],
) -> Option<&'c GeneratorOption> {
    for item in options {
        match item {
            GeneratorOptionItem::Category(category) => {
                let found_option = find_option(option, &category.options);
                if found_option.is_some() {
                    return found_option;
                }
            }
            GeneratorOptionItem::Option(item) => {
                if item.name == option {
                    return Some(item);
                }
            }
        }
    }
    None
}
