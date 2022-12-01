use core::result::Result;
use script_utils::{error::Error, nft::Nft};

type Nfts = (Nft, Nft);

pub fn validate_immutable_nft_fields((input_nft, output_nft): &Nfts) -> Result<(), Error> {
  if input_nft.data != output_nft.data {
    return Err(Error::NFTDataNotSame);
  }
  if input_nft.version != output_nft.version {
    return Err(Error::NFTVersionNotSame);
  }
  Ok(())
}
