use crate::domain::Report;

pub trait Renderer: Send + Sync {
    fn render(&self, report: &Report) -> String;
}
