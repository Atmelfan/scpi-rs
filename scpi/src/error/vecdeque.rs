use alloc::collections::vec_deque::VecDeque;
use crate::error::{Error, ErrorCode, ErrorQueue};

impl ErrorQueue for VecDeque<Error> {
    fn push_back_error(&mut self, err: Error) {
        self.push_back(err)
    }

    fn pop_front_error(&mut self) -> Error {
        self.pop_front()
            .unwrap_or_else(|| ErrorCode::NoError.into())
    }

    fn len(&self) -> usize {
        VecDeque::len(self)
    }

    fn clear(&mut self) {
        VecDeque::clear(self)
    }
}

#[cfg(test)]
mod test_error_queue {
    use alloc::collections::vec_deque::VecDeque;
    use crate::error::{Error, ErrorCode, ErrorQueue};

    #[test]
    fn test_vecqueue() {
        // Check that errorqueue returns NoError when there are no errors
        let mut errors: VecDeque<Error> = VecDeque::new();
        errors.push_back_error(ErrorCode::Custom(1, b"One").into());
        errors.push_back_error(ErrorCode::Custom(2, b"Two").into());
        assert_eq!(
            errors.pop_front_error(),
            Error::new(ErrorCode::Custom(1, b"One"))
        );
        assert_eq!(
            errors.pop_front_error(),
            Error::new(ErrorCode::Custom(2, b"Two"))
        );
        assert_eq!(
            errors.pop_front_error(),
            Error::new(ErrorCode::NoError)
        );
    }
}