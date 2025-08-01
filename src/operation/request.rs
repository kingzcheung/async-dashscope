
/// A trait for abstracting over different DashScope request parameter types.
///
/// This allows for common handling of validation and other pre-flight checks
/// before sending a request to the API.
pub trait RequestTrait {
    type P;
    /// Returns the model name for this request.
    fn model(&self) -> &str;

    /// Returns a reference to the optional parameters for this request.
    fn parameters(&self) -> Option<&Self::P>;

}
