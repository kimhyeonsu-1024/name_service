use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

/// 이름 레지스트리 프로그램에서 사용할 명령(instruction) 열거형
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum NameRegistryInstruction {
    /// Create: 이름 레코드를 생성
    /// 기대하는 계정들:
    /// 0. '[]' 시스템 프로그램(System Program)
    /// 1. '[writable, signer]' 자금 제공자(Funding) 계정
    /// 2. '[writable]' 생성될 이름 레코드 계정
    /// 3. '[]' 소유자(Owner) 계정
    /// 4. '[signer]' 클래스(Class) 계정 (선택)
    /// 5. '[]' 부모 이름 레코드(Parent) 계정 (선택)
    /// 6. '[signer]' 부모 이름 레코드의 소유자(Parent Owner) 계정 (선택)
    Create {
        hashed_name: Vec<u8>, // 해시된 이름
        lamports: u64,        // 계정에 할당할 램포트
        space: u32,           // 계정 데이터 공간
    },

    /// Update: 이름 레코드 일부 데이터를 수정
    /// 기대하는 계정들(케이스별):
    /// Case 1
    /// 0. '[writable]' 수정될 이름 레코드
    /// 1. '[signer]' 계정 소유자
    ///
    /// Case 2
    /// 0. '[writable]' 수정될 이름 레코드
    /// 1. '[signer]' 클래스 계정
    ///
    /// Case 3
    /// 0. '[writable]' 수정될 이름 레코드
    /// 1. '[signer]' 부모 이름 계정의 소유자
    /// 2. '[]' 부모 이름 레코드
    Update {
        offset: u32,    // 업데이트 시작 오프셋
        data: Vec<u8>,  // 덮어쓸 데이터
    },

    /// Transfer: 이름 레코드의 소유자를 변경
    /// 기대하는 계정들(케이스별):
    /// Case 1
    /// 0. '[writable]' 변경할 이름 레코드
    /// 1. '[signer]' 계정 소유자
    ///
    /// Case 2
    /// 0. '[writable]' 변경할 이름 레코드
    /// 1. '[signer]' 계정 소유자
    /// 2. '[signer]' 클래스 계정
    ///
    /// Case 3
    /// 0. '[writable]' 변경할 이름 레코드
    /// 1. '[signer]' 계정 소유자
    /// 2. '[]' 부모 이름 계정의 소유자
    /// 3. '[]' 부모 이름 레코드
    Transfer {
        new_owner: Pubkey, // 새 소유자
    },

    /// Delete: 이름 레코드를 삭제하고 램포트를 환불
    /// 기대하는 계정들:
    /// 0. '[writable]' 삭제할 이름 레코드
    /// 1. '[signer]' 계정 소유자
    /// 2. '[writable]' 환불 받을 계정
    Delete,
}

/// Create 명령용 Instruction 생성기
pub fn create(
    name_service_program_id: Pubkey,          // 프로그램 ID
    instruction_data: NameRegistryInstruction, // 직렬화할 명령 데이터 (Create 변형이어야 함)
    name_account_key: Pubkey,                  // 생성될 이름 계정
    name_owner: Pubkey,                        // 소유자 계정
    payer_key: Pubkey,                         // 자금 제공자(Funding) 계정
    name_class_opt: Option<Pubkey>,            // 선택적 클래스 계정(서명자)
    name_parent_opt: Option<Pubkey>,           // 선택적 부모 이름 레코드(서명자 아님)
    name_parent_owner_opt: Option<Pubkey>,     // 선택적 부모 소유자(서명자)
) -> Result<Instruction, ProgramError> {
    // Borsh 직렬화
    let data = instruction_data.try_to_vec().unwrap();

    // 기본 계정들(0~3)
    let mut accounts: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(system_program::id(), false), // 0. 시스템 프로그램
        AccountMeta::new(payer_key, true),                      // 1. 자금 제공자 (writable, signer)
        AccountMeta::new(name_account_key, true),               // 2. 이름 레코드 (writable) ※ 필요 시 signer 여부 조정
        AccountMeta::new_readonly(name_owner, false),           // 3. 소유자
    ];

    // 4. 클래스 계정(선택, 서명자)
    if let Some(name_class) = name_class_opt {
        accounts.push(AccountMeta::new_readonly(name_class, true));
    } else {
        // 슬롯을 고정해야 한다면 placeholder 추가
        accounts.push(AccountMeta::new_readonly(Pubkey::default(), false));
    }

    // 5. 부모 이름 레코드(선택, 서명자 아님)
    if let Some(name_parent) = name_parent_opt {
        accounts.push(AccountMeta::new_readonly(name_parent, false));
    } else {
        accounts.push(AccountMeta::new_readonly(Pubkey::default(), false));
    }

    // 6. 부모 소유자(선택, 서명자)
    if let Some(parent_owner) = name_parent_owner_opt {
        accounts.push(AccountMeta::new_readonly(parent_owner, true));
    } else {
        accounts.push(AccountMeta::new_readonly(Pubkey::default(), false));
    }

    Ok(Instruction {
        program_id: name_service_program_id,
        accounts,
        data,
    })
}

/// Update 명령용 Instruction 생성기
pub fn update(
    name_service_program_id: Pubkey, // 프로그램 ID
    offset: u32,                     // 수정 시작 오프셋
    data: Vec<u8>,                   // 덮어쓸 데이터
    name_account_key: Pubkey,        // 대상 이름 레코드
    name_update_signer: Pubkey,      // 업데이트 권한 서명자(Owner/Class/Parent Owner 중 하나)
    name_parent_opt: Option<Pubkey>, // Case 3에서 필요한 부모 이름 레코드
) -> Result<Instruction, ProgramError> {
    // 명령 데이터 구성 및 직렬화
    let instruction_data = NameRegistryInstruction::Update { offset, data };
    let data = instruction_data.try_to_vec().unwrap();

    // 기본 계정들
    let mut accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(name_account_key, false),   // [writable] 이름 레코드
        AccountMeta::new(name_update_signer, true),  // [signer] 업데이트 권한자
    ];

    // Case 3: 부모 이름 레코드가 필요한 경우
    if let Some(name_parent_key) = name_parent_opt {
        accounts.push(AccountMeta::new_readonly(name_parent_key, false)); // 부모 레코드는 서명자 아님
    }

    Ok(Instruction {
        program_id: name_service_program_id,
        accounts,
        data,
    })
}

pub fn transfer(
    name_service_program_id: Pubkey,
    name_owner: Pubkey,            // 새 소유자 (Transfer 데이터에 들어감)
    name_account_key: Pubkey,      // 이름 레코드 (writable)
    name_owner_key: Pubkey,        // 현재 소유자 (signer)
    name_parent: Option<Pubkey>,   // 선택적 부모 레코드
) -> Result<Instruction, ProgramError> {
    // 올바른 값 생성 구문 (타입 지정 X)
    let instruction_data = NameRegistryInstruction::Transfer { new_owner: name_owner };
    let data = instruction_data.try_to_vec().unwrap();

    let mut accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(name_account_key, false),        // [writable]
        AccountMeta::new_readonly(name_owner_key, true),  // [signer]
    ];

    // 선택적 부모 레코드가 있는 경우 추가 (부모 레코드는 signer 아님)
    if let Some(parent) = name_parent {
        accounts.push(AccountMeta::new_readonly(parent, false));
    }

    Ok(Instruction {
        program_id: name_service_program_id,
        accounts,
        data,
    })
}

pub fn delete(
    name_service_program_id: Pubkey,
    name_account_key: Pubkey,
    name_owner_key: Pubkey,
    refund_target: Pubkey,
) -> Result<Instruction, ProgramError> {
    let instruction_data = NameRegistryInstruction::Delete;
    let data = instruction_data.try_to_vec().unwrap();
    let accounts = vec![
        AccountMeta::new(name_account_key, false),
        AccountMeta::new_readonly(name_owner_key, true),
        AccountMeta::new(refund_target, false),
    ];

    Ok(Instruction {
        program_id: name_service_program_id,
        accounts,
        data,
    })
}