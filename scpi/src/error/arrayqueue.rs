use crate::error::{Error, ErrorCode, ErrorQueue};
use arrayvec::ArrayVec;
use core::default::Default;

/// Default error queue based on a alloc-less arrayqueue.
pub struct ArrayErrorQueue<const CAP: usize> {
    vec: ArrayVec<Error, CAP>,
}

impl<const CAP: usize> Default for ArrayErrorQueue<CAP> {
    fn default() -> Self {
        ArrayErrorQueue {
            vec: ArrayVec::new(),
        }
    }
}

impl<const CAP: usize> ArrayErrorQueue<CAP> {
    pub fn new() -> Self {
        ArrayErrorQueue::default()
    }
}

impl<const CAP: usize> ErrorQueue for ArrayErrorQueue<CAP> {
    fn push_back_error(&mut self, err: Error) {
        //Try to queue an error, replace last with QueueOverflow if full
        if self.vec.try_push(err).is_err() {
            let _ = self.vec.pop().unwrap();
            self.vec.try_push(ErrorCode::QueueOverflow.into()).unwrap();
        }
    }

    fn pop_front_error(&mut self) -> Error {
        self.vec
            .pop_at(0)
            .unwrap_or_else(|| ErrorCode::NoError.into())
    }

    fn len(&self) -> usize {
        self.vec.len()
    }

    fn clear(&mut self) {
        self.vec.clear()
    }
}

#[cfg(test)]
mod test_error_queue {
    use super::*;
    use crate::error::{Error, ErrorCode, ErrorQueue};

    #[test]
    fn test_extended() {
        // Check that errorqueue returns NoError when there are no errors
        let mut errors = ArrayErrorQueue::<10>::new();
        errors.push_back_error(Error::extended(ErrorCode::Custom(1, b"Error"), b"Extended"));
        #[cfg(feature = "extended-error")]
        assert_eq!(
            errors.pop_front_error(),
            Error(ErrorCode::Custom(1, b"Error"), Some(b"Extended"))
        );
        #[cfg(not(feature = "extended-error"))]
        assert_eq!(
            errors.pop_front_error(),
            Error(ErrorCode::Custom(1, b"Error"))
        );
    }

    #[test]
    fn test_queue_noerror() {
        // Check that errorqueue returns NoError when there are no errors
        let mut errors = ArrayErrorQueue::<10>::new();
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
        assert_eq!(errors.pop_front_error(), Error::new(ErrorCode::NoError));
    }

    #[test]
    fn test_queue_overflow() {
        // Check that errorqueue returns NoError when there are no errors
        let mut errors = ArrayErrorQueue::<2>::new();
        errors.push_back_error(ErrorCode::Custom(1, b"One").into());
        errors.push_back_error(ErrorCode::Custom(2, b"Two").into());
        errors.push_back_error(ErrorCode::Custom(3, b"Three").into());
        assert_eq!(
            errors.pop_front_error(),
            Error::new(ErrorCode::Custom(1, b"One"))
        );
        assert_eq!(
            errors.pop_front_error(),
            Error::new(ErrorCode::QueueOverflow)
        );
    }
}
