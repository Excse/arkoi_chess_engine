use std::borrow::Cow;

use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus};

pub static DEFAULT_MULTILINE_INDICATOR: &str = ":: ";
pub static DEFAULT_PROMPT_INDICATOR: &str = "> ";
pub static DEFAULT_PROMPT_RIGHT: &str = "";
pub static DEFAULT_PROMPT_LEFT: &str = "";

#[derive(Default, Clone)]
pub struct CustomPrompt;

impl Prompt for CustomPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_PROMPT_LEFT)
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_PROMPT_RIGHT)
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        Cow::Borrowed(DEFAULT_PROMPT_INDICATOR)
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}
