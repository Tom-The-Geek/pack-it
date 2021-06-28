use rustyline::Editor;
use anyhow::Result;
use colored::*;

pub struct QuestionAsker {
    editor: Editor<()>,
}

impl QuestionAsker {
    pub fn new() -> Self {
        Self {
            editor: Editor::<()>::new(),
        }
    }

    pub fn ask_question(&mut self, question: &str) -> Result<String> {
        let res = self.editor.readline(&*format!("{} {}",
                                                 question.bright_blue(),
                                                 ">>> ".yellow()
        ))?;
        Ok(res)
    }

    pub fn ask_from_list(&mut self, question: &str, options: &[&str]) -> Result<String> {
        let mut answer = self.ask_question(question)?;
        while !options.contains(&answer.as_str()) {
            answer = self.ask_question(question)?;
        }
        Ok(answer)
    }
}
