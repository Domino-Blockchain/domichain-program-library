use {
    domichain_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        msg,
        program::invoke,
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
    },
    std::convert::TryInto,
};

#[inline(always)]
pub fn create_new_account<'a>(
    from_info: &AccountInfo<'a>,
    new_account_info: &AccountInfo<'a>,
    space: usize,
    owner_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_info)?;
    let required_satomis = rent
        .minimum_balance(space)
        .max(1)
        .saturating_sub(new_account_info.satomis());

    msg!("Transfer {} satomis to the new account", required_satomis);
    invoke(
        &system_instruction::create_account(
            from_info.key,
            new_account_info.key,
            required_satomis,
            space as u64,
            owner_info.key,
        ),
        &[from_info.clone(), new_account_info.clone()],
    )?;
    Ok(())
}

#[inline(always)]
pub fn topup<'a>(
    account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_satomis = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(account_info.satomis());

    if required_satomis > 0 {
        msg!("Transfer {} satomis to the new account", required_satomis);
        invoke(
            &system_instruction::transfer(payer_info.key, account_info.key, required_satomis),
            &[
                payer_info.clone(),
                account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }
    Ok(())
}

#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
) -> ProgramResult {
    topup(
        new_account_info,
        rent_sysvar_info,
        system_program_info,
        payer_info,
        size,
    )?;
    msg!("Allocate space for the account");
    invoke(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        &[new_account_info.clone(), system_program_info.clone()],
    )?;

    msg!("Assign the account to the owning program");
    invoke(
        &system_instruction::assign(new_account_info.key, &program_id),
        &[new_account_info.clone(), system_program_info.clone()],
    )?;

    Ok(())
}
