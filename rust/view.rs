use rivulet::{circular_buffer as cbuf, View, ViewMut};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Error {
    fn from_error<T: std::fmt::Display>(e: T) -> Self {
        Self(format!("{}", e))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.0
    }
}

type MapError<T> = rivulet::view::MapError<T, Error, fn(<T as rivulet::View>::Error) -> Error>;
type BufInput = MapError<rivulet::circular_buffer::Sink<u8>>;
type BufOutput = MapError<rivulet::circular_buffer::Source<u8>>;

trait PipelineImpl {
    fn to_stream(&mut self) -> Option<Box<dyn Stream>>;

    fn split(&mut self) -> Option<(Box<dyn PipelineImpl>, Box<dyn PipelineImpl>)>;
}

impl<T> PipelineImpl for Option<T>
where
    T: 'static + rivulet::SplittableView<Item = u8>,
    T::Error: 'static + std::fmt::Display,
{
    fn to_stream(&mut self) -> Option<Box<dyn Stream>> {
        if let Some(p) = self.take() {
            Some(Box::new(
                p.into_cloneable_view().map_error(Error::from_error),
            ))
        } else {
            None
        }
    }

    fn split(&mut self) -> Option<(Box<dyn PipelineImpl>, Box<dyn PipelineImpl>)> {
        if let Some(p) = self.take() {
            let (first, second) = p.sequence();
            Some((Box::new(Some(first)), Box::new(Some(second))))
        } else {
            None
        }
    }
}

/// A type-erased pipeline component.
struct Pipeline(Box<dyn PipelineImpl>);

impl Pipeline {
    fn to_stream(&mut self) -> Option<Box<dyn Stream>> {
        self.0.to_stream()
    }

    fn precede(&mut self) -> Option<Self> {
        if let Some((first, second)) = self.0.split() {
            self.0 = second;
            Some(Self(first))
        } else {
            None
        }
    }

    fn follow(&mut self) -> Option<Self> {
        if let Some((first, second)) = self.0.split() {
            self.0 = first;
            Some(Self(second))
        } else {
            None
        }
    }
}

trait Stream: rivulet::View<Item = u8, Error = Error> {
    fn duplicate(&self) -> Box<dyn Stream>;
}

impl<T> Stream for T
where
    T: 'static + rivulet::View<Item = u8, Error = Error> + Clone,
{
    fn duplicate(&self) -> Box<dyn Stream> {
        Box::new(self.clone())
    }
}
