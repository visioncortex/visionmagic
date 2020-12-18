/// Processor is an element of an image processing pipeline
pub trait Processor {

    /// Type definition of input
    type Input;
    /// Type definition of output
    type Output;
    /// Type definition of parameters
    type Params;

    /// Create a new Processor instance
    fn new() -> Self;

    /// Configure parameters; returns true for valid config
    fn config(&mut self, params: Self::Params) -> bool;

    /// Provide input to Processor; returns true for valid input
    fn input(&mut self, input: Self::Input) -> bool;

    /// Handover control to Processor to perform one unit of work; returns true when finished
    fn tick(&mut self) -> bool;

    /// Check progress; returns an integer from 0 to 100 (inclusive)
    fn progress(&self) -> u32;

    /// Retrieve output from Processor
    fn output(&mut self) -> Self::Output;

}