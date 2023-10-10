use rivulet::{
    splittable::{SplittableView, SplittableViewMut},
    View, ViewMut,
};

#[derive(Debug)]
pub struct Error(String);

impl Error {
    fn new(message: &str) -> Self {
        Self(message.to_string())
    }
}

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

pub trait PipeImpl {
    fn tap(&mut self) -> Option<Box<dyn Tap>>;

    fn split(&mut self) -> Option<(Box<dyn PipeImpl>, Box<dyn PipeImpl>)>;
}

pub trait PipeMutImpl: PipeImpl {
    fn tap_mut(&mut self) -> Option<Box<dyn TapMut>>;

    fn split_mut(&mut self) -> Option<(Box<dyn PipeMutImpl>, Box<dyn PipeMutImpl>)>;
}

impl<T> PipeImpl for Option<T>
where
    T: 'static + SplittableView<Item = u8>,
    T::Error: 'static + std::fmt::Display,
{
    fn tap(&mut self) -> Option<Box<dyn Tap>> {
        if let Some(p) = self.take() {
            Some(Box::new(
                p.into_cloneable_view().map_error(Error::from_error),
            ))
        } else {
            None
        }
    }

    fn split(&mut self) -> Option<(Box<dyn PipeImpl>, Box<dyn PipeImpl>)> {
        if let Some(p) = self.take() {
            let (first, second) = p.sequence();
            Some((Box::new(Some(first)), Box::new(Some(second))))
        } else {
            None
        }
    }
}

impl<T> PipeMutImpl for Option<T>
where
    T: 'static + SplittableViewMut<Item = u8>,
    T::Error: 'static + std::fmt::Display,
{
    fn tap_mut(&mut self) -> Option<Box<dyn TapMut>> {
        if let Some(p) = self.take() {
            Some(Box::new(p.into_view().map_error(Error::from_error)))
        } else {
            None
        }
    }

    fn split_mut(&mut self) -> Option<(Box<dyn PipeMutImpl>, Box<dyn PipeMutImpl>)> {
        if let Some(p) = self.take() {
            let (first, second) = p.sequence();
            Some((Box::new(Some(first)), Box::new(Some(second))))
        } else {
            None
        }
    }
}

/// A type-erased pipeline component.
pub enum Pipe {
    Const(Box<dyn PipeImpl>),
    Mut(Box<dyn PipeMutImpl>),
}

impl Pipe {
    pub fn tap(&mut self) -> Result<Box<dyn Tap>, Error> {
        let s = match self {
            Self::Const(p) => p.tap(),
            Self::Mut(p) => p.tap(),
        };
        s.ok_or_else(|| Error::new("a stream has already been created from this pipe"))
    }

    pub fn mutable_tap(&mut self) -> Result<Box<dyn TapMut>, Error> {
        let s = match self {
            Self::Const(_) => return Err(Error::new("this pipe is not mutable")),
            Self::Mut(p) => p.tap_mut(),
        };
        s.ok_or_else(|| Error::new("a stream has already been created from this pipe"))
    }

    pub fn split(&mut self) -> Option<(Self, Self)> {
        match self {
            Self::Const(p) => p
                .split()
                .map(|(first, second)| (Self::Const(first), Self::Const(second))),
            Self::Mut(p) => p
                .split_mut()
                .map(|(first, second)| (Self::Mut(first), Self::Mut(second))),
        }
    }
}

pub trait Tap: rivulet::View<Item = u8, Error = Error> {
    fn duplicate(&self) -> Box<dyn Tap>;
}

impl<T> Tap for T
where
    T: 'static + View<Item = u8, Error = Error> + Clone,
{
    fn duplicate(&self) -> Box<dyn Tap> {
        Box::new(self.clone())
    }
}

pub trait TapMut: ViewMut<Item = u8, Error = Error> {}

impl<T> TapMut for T where T: 'static + rivulet::ViewMut<Item = u8, Error = Error> {}
