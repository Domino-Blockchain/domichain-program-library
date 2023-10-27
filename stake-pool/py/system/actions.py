from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Confirmed


async def airdrop(client: AsyncClient, receiver: PublicKey, satomis: int):
    print(f"Airdropping {satomis} satomis to {receiver}...")
    resp = await client.request_airdrop(receiver, satomis, Confirmed)
    await client.confirm_transaction(resp['result'], Confirmed)
