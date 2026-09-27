use db::tables::{FeedParsingRuleRow, RuleTargetField};
use regex::Regex;

pub enum TargetField {
    Title,
    Description,
}

pub struct CompiledRule {
    pub regex: Regex,
    pub target_field: TargetField,
    pub capture_group: usize,
}

impl CompiledRule {
    // Extracts all possible guest matching the our pattern
    pub fn extract(&self, title: &str, description: &str) -> Vec<String> {
        let text = self.field_text(title, description);
        self.regex.captures_iter(text)
            .filter_map(|caps| caps.get(self.capture_group).map(|m| m.as_str().trim().to_string()))
            .collect()
    }

    fn field_text<'a>(&self, title: &'a str, description: &'a str) -> &'a str {
        match self.target_field {
            TargetField::Title => title,
            TargetField::Description => description,
        }
    }
}

pub fn compile_rules(rows: Vec<FeedParsingRuleRow>) -> Result<Vec<CompiledRule>, regex::Error> {
    rows.into_iter()
        .map(|row| {
            Ok(CompiledRule {
                regex: Regex::new(&row.pattern)?,
                target_field: match row.target_field {
                    RuleTargetField::Title => TargetField::Title,
                    RuleTargetField::Description => TargetField::Description,
                },
                capture_group: row.capture_group.max(0) as usize,
            })
        })
        .collect()
}
