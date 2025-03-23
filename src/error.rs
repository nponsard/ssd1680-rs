#[derive(Debug)]
pub enum Error<S, R, D, B> {
    SpiError(S),
    RstPinError(R),
    DcPinError(D),
    BusyPinError(B),
}
