import argparse
import asyncio
import json

from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Confirmed

from stake.constants import STAKE_LEN, SATOMIS_PER_SOL
from stake_pool.actions import decrease_validator_stake, increase_validator_stake, update_stake_pool
from stake_pool.constants import MINIMUM_ACTIVE_STAKE
from stake_pool.state import StakePool, ValidatorList


async def get_client(endpoint: str) -> AsyncClient:
    print(f'Connecting to network at {endpoint}')
    async_client = AsyncClient(endpoint=endpoint, commitment=Confirmed)
    total_attempts = 10
    current_attempt = 0
    while not await async_client.is_connected():
        if current_attempt == total_attempts:
            raise Exception("Could not connect to test validator")
        else:
            current_attempt += 1
        await asyncio.sleep(1)
    return async_client


async def rebalance(endpoint: str, stake_pool_address: PublicKey, staker: Keypair, retained_reserve_amount: float):
    async_client = await get_client(endpoint)

    resp = await async_client.get_epoch_info(commitment=Confirmed)
    epoch = resp['result']['epoch']
    resp = await async_client.get_account_info(stake_pool_address, commitment=Confirmed)
    data = resp['result']['value']['data']
    stake_pool = StakePool.decode(data[0], data[1])

    print(f'Stake pool last update epoch {stake_pool.last_update_epoch}, current epoch {epoch}')
    if stake_pool.last_update_epoch != epoch:
        print('Updating stake pool')
        await update_stake_pool(async_client, staker, stake_pool_address)
        resp = await async_client.get_account_info(stake_pool_address, commitment=Confirmed)
        data = resp['result']['value']['data']
        stake_pool = StakePool.decode(data[0], data[1])

    resp = await async_client.get_minimum_balance_for_rent_exemption(STAKE_LEN)
    stake_rent_exemption = resp['result']
    retained_reserve_satomis = int(retained_reserve_amount * SATOMIS_PER_SOL)

    resp = await async_client.get_account_info(stake_pool.validator_list, commitment=Confirmed)
    data = resp['result']['value']['data']
    validator_list = ValidatorList.decode(data[0], data[1])

    print('Stake pool stats:')
    print(f'* {stake_pool.total_satomis} total satomis')
    num_validators = len(validator_list.validators)
    print(f'* {num_validators} validators')
    print(f'* Retaining {retained_reserve_satomis} satomis in the reserve')
    satomis_per_validator = (stake_pool.total_satomis - retained_reserve_satomis) // num_validators
    num_increases = sum([
        1 for validator in validator_list.validators
        if validator.transient_stake_satomis == 0 and validator.active_stake_satomis < satomis_per_validator
    ])
    total_usable_satomis = stake_pool.total_satomis - retained_reserve_satomis - num_increases * stake_rent_exemption
    satomis_per_validator = total_usable_satomis // num_validators
    print(f'* {satomis_per_validator} satomis desired per validator')

    futures = []
    for validator in validator_list.validators:
        if validator.transient_stake_satomis != 0:
            print(f'Skipping {validator.vote_account_address}: {validator.transient_stake_satomis} transient satomis')
        else:
            if validator.active_stake_satomis > satomis_per_validator:
                satomis_to_decrease = validator.active_stake_satomis - satomis_per_validator
                if satomis_to_decrease <= stake_rent_exemption:
                    print(f'Skipping decrease on {validator.vote_account_address}, \
currently at {validator.active_stake_satomis} satomis, \
decrease of {satomis_to_decrease} below the rent exmption')
                else:
                    futures.append(decrease_validator_stake(
                        async_client, staker, staker, stake_pool_address,
                        validator.vote_account_address, satomis_to_decrease
                    ))
            elif validator.active_stake_satomis < satomis_per_validator:
                satomis_to_increase = satomis_per_validator - validator.active_stake_satomis
                if satomis_to_increase < MINIMUM_ACTIVE_STAKE:
                    print(f'Skipping increase on {validator.vote_account_address}, \
currently at {validator.active_stake_satomis} satomis, \
increase of {satomis_to_increase} less than the minimum of {MINIMUM_ACTIVE_STAKE}')
                else:
                    futures.append(increase_validator_stake(
                        async_client, staker, staker, stake_pool_address,
                        validator.vote_account_address, satomis_to_increase
                    ))
            else:
                print(f'{validator.vote_account_address}: already at {satomis_per_validator}')

    print('Executing strategy')
    await asyncio.gather(*futures)
    print('Done')
    await async_client.close()


def keypair_from_file(keyfile_name: str) -> Keypair:
    with open(keyfile_name, 'r') as keyfile:
        data = keyfile.read()
    int_list = json.loads(data)
    bytes_list = [value.to_bytes(1, 'little') for value in int_list]
    return Keypair.from_secret_key(b''.join(bytes_list))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Rebalance stake evenly between all the validators in a stake pool.')
    parser.add_argument('stake_pool', metavar='STAKE_POOL_ADDRESS', type=str,
                        help='Stake pool to rebalance, given by a public key in base-58,\
                         e.g. Zg5YBPAk8RqBR9kaLLSoN5C8Uv7nErBz1WC63HTsCPR')
    parser.add_argument('staker', metavar='STAKER_KEYPAIR', type=str,
                        help='Staker for the stake pool, given by a keypair file, e.g. staker.json')
    parser.add_argument('reserve_amount', metavar='RESERVE_AMOUNT', type=float,
                        help='Amount of SOL to keep in the reserve, e.g. 10.5')
    parser.add_argument('--endpoint', metavar='ENDPOINT_URL', type=str,
                        default='https://api.mainnet-beta.solana.com',
                        help='RPC endpoint to use, e.g. https://api.mainnet-beta.solana.com')

    args = parser.parse_args()
    stake_pool = PublicKey(args.stake_pool)
    staker = keypair_from_file(args.staker)
    print(f'Rebalancing stake pool {stake_pool}')
    print(f'Staker public key: {staker.public_key}')
    print(f'Amount to leave in the reserve: {args.reserve_amount} SOL')
    asyncio.run(rebalance(args.endpoint, stake_pool, staker, args.reserve_amount))
