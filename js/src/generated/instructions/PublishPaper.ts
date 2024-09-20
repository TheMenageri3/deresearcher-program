/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'

/**
 * @category Instructions
 * @category PublishPaper
 * @category generated
 */
export const PublishPaperStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number
}>([['instructionDiscriminator', beet.u8]], 'PublishPaperInstructionArgs')
/**
 * Accounts required by the _PublishPaper_ instruction
 *
 * @property [_writable_, **signer**] publisherAcc
 * @property [_writable_] paperPdaAcc
 * @category Instructions
 * @category PublishPaper
 * @category generated
 */
export type PublishPaperInstructionAccounts = {
  publisherAcc: web3.PublicKey
  paperPdaAcc: web3.PublicKey
}

export const publishPaperInstructionDiscriminator = 2

/**
 * Creates a _PublishPaper_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category PublishPaper
 * @category generated
 */
export function createPublishPaperInstruction(
  accounts: PublishPaperInstructionAccounts,
  programId = new web3.PublicKey('P1SsZEQvb6gTPrdJQ5mu6oCyJCJhVKxFFnk9ztjsoEL')
) {
  const [data] = PublishPaperStruct.serialize({
    instructionDiscriminator: publishPaperInstructionDiscriminator,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.publisherAcc,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.paperPdaAcc,
      isWritable: true,
      isSigner: false,
    },
  ]

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}
