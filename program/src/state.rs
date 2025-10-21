use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::cell::RefMut;
use std::io::Cursor;

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct NameRecordHeader {
    pub parent_name: Pubkey,
    pub owner: Pubkey,
    pub class: Pubkey,
}

impl Sealed for NameRecordHeader {}

impl Pack for NameRecordHeader {
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

pub fn write_data(account_: &AccountInfo, input: &[u8], offset: usize) {
    let mut account_data: RefMut<&mut [u8]> = account_.data.borrow_mut();
    account_data[offset..offset + input.len()].copy_from_slice(input);
}

pub fn get_seeds_and_key(
    program_id: &Pubkey,
    hashed_name: Vec<u8>,
    name_class_opt: Option<&Pubkey>,
    parent_name_address_opt: Option<&Pubkey>,
) -> (Pubkey, Vec<u8>) {
    let mut seeds_vec: Vec<u8> = hashed_name;

    let name_class: Pubkey = name_class_opt.cloned().unwrap_or_default();
    seeds_vec.extend_from_slice(name_class.as_ref());

    let parent_name_address: Pubkey = parent_name_address_opt.cloned().unwrap_or_default();
    seeds_vec.extend_from_slice(parent_name_address.as_ref());

    let seed_slices: Vec<&[u8]> = seeds_vec.chunks(32).collect();
    let (name_account_key, bump) = Pubkey::find_program_address(&seed_slices, program_id);
    seeds_vec.push(bump);

    (name_account_key, seeds_vec)
}