use winnow::{
    ModalResult,
    error::{ContextError, ErrMode, StrContext},
};

pub fn error<O>(msg: &'static str) -> ModalResult<O> {
    let mut err = ContextError::new();
    err.push(StrContext::Label(msg));
    Err(ErrMode::Cut(err))
}
