use ckb_std::error::SysError;

/// Error
#[repr(i8)]
pub enum Error {
  IndexOutOfBound = 1,
  ItemMissing,
  LengthNotEnough,
  Encoding,
  IssuerDataInvalid = 5,
  IssuerCellsCountError,
  TypeArgsInvalid,
  IssuerClassCountError,
  IssuerCellCannotDestroyed,
  VersionInvalid=10,
  ClassDataInvalid,
  ClassTotalSmallerThanIssued,
  ClassCellsCountError,
  ClassIssuedInvalid,
  ClassImmutableFieldsNotSame = 15,
  ClassCellCannotDestroyed,
  ClassIdIncreaseError,
  NFTDataInvalid,
  NFTCellsCountError,
  NFTDataNotSame = 20,
  GroupInputWitnessNoneError,
  MetadataCellsCountError,
  MetadataImmutableFieldsNotSame,
  MetadataDataInvalid,
  MetadataIdIncreaseError = 25,
  PaymentNotEnough,
  InvalidPaymentLockScript,
  NFTVersionNotSame,
  NFTVersionNotSameWithClass,
  CannotDecrementClassVersion = 30,
}

impl From<SysError> for Error {
  fn from(err: SysError) -> Self {
    use SysError::*;
    match err {
      IndexOutOfBound => Self::IndexOutOfBound,
      ItemMissing => Self::ItemMissing,
      LengthNotEnough(_) => Self::LengthNotEnough,
      Encoding => Self::Encoding,
      Unknown(err_code) => panic!("unexpected sys error {}", err_code),
    }
  }
}