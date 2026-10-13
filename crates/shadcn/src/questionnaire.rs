//! A native multi-step questionnaire with retained answers and explicit validation.
use crate::{Button, Variant, column, row};
#[derive(Debug, Clone)]
pub struct QuestionChoice {
    pub label: String,
    pub description: String,
}
#[derive(Debug, Clone)]
pub struct Question {
    pub name: String,
    pub title: String,
    pub description: String,
    pub choices: Vec<QuestionChoice>,
    pub required: bool,
    pub multiple: bool,
    pub allow_other: bool,
}
#[derive(Debug, Clone, Default)]
pub struct Answer {
    pub choices: Vec<usize>,
    pub other: String,
    pub skipped: bool,
}
#[derive(Debug, Clone, Default)]
pub struct QuestionnaireState {
    pub current: usize,
    pub answers: Vec<Answer>,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct QuestionnaireResponse {
    pub submitted: bool,
    pub changed: bool,
}
impl QuestionnaireState {
    pub fn valid(&self, index: usize, question: &Question) -> bool {
        !question.required
            || self
                .answers
                .get(index)
                .is_some_and(|answer| !answer.choices.is_empty() || !answer.other.trim().is_empty())
    }
}
/// Previous/skip/next/submit navigation preserves every answer. The application consumes
/// `submitted` and performs its own action; this component sends no network requests.
#[track_caller]
pub fn questionnaire(
    questions: &[Question],
    state: &mut QuestionnaireState,
) -> QuestionnaireResponse {
    let mut response = QuestionnaireResponse::default();
    if questions.is_empty() {
        return response;
    }
    state.answers.resize_with(questions.len(), Answer::default);
    state.current = state.current.min(questions.len() - 1);
    let question = &questions[state.current];
    column(16., || {
        crate::muted(format!(
            "Question {} of {}",
            state.current + 1,
            questions.len()
        ));
        crate::card_title(&question.title);
        crate::muted(&question.description);
        for (i, choice) in question.choices.iter().enumerate() {
            let selected = state.answers[state.current].choices.contains(&i);
            crate::item(
                if selected {
                    Variant::Secondary
                } else {
                    Variant::Outline
                },
                || {
                    column(4., || {
                        if Button::new(&choice.label)
                            .variant(Variant::Ghost)
                            .show()
                            .clicked
                        {
                            let answer = &mut state.answers[state.current];
                            if question.multiple {
                                if selected {
                                    answer.choices.retain(|j| *j != i);
                                } else {
                                    answer.choices.push(i);
                                }
                            } else {
                                answer.choices = vec![i];
                            }
                            answer.other.clear();
                            answer.skipped = false;
                            response.changed = true;
                        }
                        if !choice.description.is_empty() {
                            crate::muted(&choice.description);
                        }
                    })
                },
            );
        }
        if question.allow_other {
            if crate::input(
                &mut state.answers[state.current].other,
                "Describe another answer...",
            ) {
                state.answers[state.current].choices.clear();
                response.changed = true;
            }
        }
        if let Some(error) = &state.error {
            let t = crate::theme();
            yakui::widgets::RenderText::with_style(error, t.text_style(14., t.destructive, 400))
                .show();
        }
        row(8., || {
            if Button::new("Previous")
                .variant(Variant::Outline)
                .disabled(state.current == 0)
                .show()
                .clicked
            {
                state.current = state.current.saturating_sub(1);
                state.error = None;
            }
            if Button::new("Skip")
                .variant(Variant::Ghost)
                .disabled(question.required)
                .show()
                .clicked
            {
                state.answers[state.current].skipped = true;
                state.current = (state.current + 1).min(questions.len() - 1);
                state.error = None;
            }
            if Button::new(if state.current + 1 == questions.len() {
                "Submit"
            } else {
                "Next"
            })
            .show()
            .clicked
            {
                if !state.valid(state.current, question) {
                    state.error = Some("Please choose an answer before continuing.".into());
                } else if state.current + 1 < questions.len() {
                    state.current += 1;
                    state.error = None;
                } else if let Some(index) = questions
                    .iter()
                    .enumerate()
                    .find_map(|(i, q)| (!state.valid(i, q)).then_some(i))
                {
                    state.current = index;
                    state.error = Some("This answer is required.".into());
                } else {
                    response.submitted = true;
                    state.error = None;
                }
            }
        });
    });
    response
}
