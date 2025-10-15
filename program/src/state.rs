use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::io::Cursor; // ✅ Borsh serialize/deserialize에 필요

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct NameRecordHeader {
    pub parent_name: Pubkey,
    pub owner: Pubkey,
    pub class: Pubkey,
}

// Pack을 쓰려면 Sealed 필요
impl Sealed for NameRecordHeader {}

impl Pack for NameRecordHeader {
    // 32바이트 * 3 = 96
    const LEN: usize = 96;

    fn pack_into_slice(&self, dst: &mut [u8]) {

        let mut writer = Cursor::new(dst);
        self.serialize(&mut writer).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        NameRecordHeader::try_from_slice(src).map_err(|_| {
            msg!("Failed to deserialize name record");
            ProgramError::InvalidAccountData
        })

     
    }
}

impl IsInitialized for NameRecordHeader {
    fn is_initialized(&self) -> bool {
        self.owner != Pubkey::default()
    }
}

pub fn write_data(account_: &AccountInfo, input: &[u8], offset: usize){
    let mut account_data: RefMut<&mut [u8]> = account.data.borrow_mut();
    account_data[offset..offset + input.len()].copy_from_slice(input);
}