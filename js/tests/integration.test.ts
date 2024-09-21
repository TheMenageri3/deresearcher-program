import * as sdk from "../src";
import fs from "fs";
import * as solana from "@solana/web3.js";
import os from "os";
import { sleep } from "bun";

const getLocalWallet = () => {
  let homeDir = os.homedir();

  const localWalletFile = fs.readFileSync(homeDir + "/.config/solana/id.json");

  let jsonParsed = Uint8Array.from(JSON.parse(localWalletFile.toString()));

  return solana.Keypair.fromSecretKey(jsonParsed);
};

const localWallet = solana.Keypair.generate();

const connection = new solana.Connection("http://127.0.0.1:8899");

console.log("Airdropping... for pubkey", localWallet.publicKey.toBase58());
const txId = await connection.requestAirdrop(
  localWallet.publicKey,
  10 * solana.LAMPORTS_PER_SOL
);

await connection.confirmTransaction(txId, "confirmed");

describe("Integration tests", () => {
  it("Create a ReseacherProfile account", async () => {
    try {
      const seeds = [
        Buffer.from("deres_profile"),
        localWallet.publicKey.toBuffer(),
      ];

      const [researcherProfilePda, bump] =
        solana.PublicKey.findProgramAddressSync(seeds, sdk.PROGRAM_ID);

      console.log("Researcher profile pda", researcherProfilePda.toBase58());

      const ix = sdk.createCreateResearcherProfileInstruction(
        {
          researcherAcc: localWallet.publicKey,
          researcherProfilePdaAcc: researcherProfilePda,
          systemProgramAcc: solana.SystemProgram.programId,
        },
        {
          createResearcherProfile: {
            name: "jack",
            pdaBump: bump,
          },
        }
      );

      const tx = new solana.Transaction().add(ix);

      const blockhashWithHeight = await connection.getLatestBlockhash();

      tx.recentBlockhash = blockhashWithHeight.blockhash;

      tx.feePayer = localWallet.publicKey;

      tx.sign(localWallet);

      const txSig = await connection.sendRawTransaction(tx.serialize(), {
        preflightCommitment: "confirmed",
      });

      console.log("Transaction signature", txSig);

      await connection.confirmTransaction(txSig, "finalized");

      console.log(txSig);
    } catch (e) {
      console.error(e);
    }
  });

  it("fetch the researcher profile", async () => {
    try {
      const seeds = [
        Buffer.from("deres_profile"),
        localWallet.publicKey.toBuffer(),
      ];

      const [researcherProfilePda, bump] =
        solana.PublicKey.findProgramAddressSync(seeds, sdk.PROGRAM_ID);

      let acc_info = await connection.getAccountInfo(researcherProfilePda);

      if (!acc_info) {
        console.error("Account not found");
        return;
      }

      const [acc, _id] =
        sdk.accountProviders.ResearcherProfile.fromAccountInfo(acc_info);
      console.log(acc.pretty());
    } catch (e) {
      console.error(e);
    }
  });

  it("Create a Research paper", async () => {
    try {
      const paperContentHash = "48y2ehidkhdkhadahkhadhiakhdiaydh"; //32 bytes

      const seeds = [
        Buffer.from("deres_paper"),
        Buffer.from(paperContentHash),
        localWallet.publicKey.toBuffer(),
      ];

      const [paperPda, bump] = solana.PublicKey.findProgramAddressSync(
        seeds,
        sdk.PROGRAM_ID
      );

      console.log("Paper pda", paperPda.toBase58());

      const researcherProfilePda = solana.PublicKey.findProgramAddressSync(
        [Buffer.from("deres_profile"), localWallet.publicKey.toBuffer()],
        sdk.PROGRAM_ID
      )[0];

      console.log("Researcher profile pda", researcherProfilePda.toBase58());

      const ix = sdk.createCreateResearchePaperInstruction(
        {
          publisherAcc: localWallet.publicKey,
          researcherProfilePdaAcc: researcherProfilePda,
          paperPdaAcc: paperPda,
          systemProgramAcc: solana.SystemProgram.programId,
        },
        {
          createResearchePaper: {
            paperContentHash: paperContentHash,
            accessFee: 100,
            metaDataMerkleRoot: "djagdbjadbjadbjaldb",
            pdaBump: bump,
          },
        }
      );

      const tx = new solana.Transaction().add(ix);

      const blockhashWithHeight = await connection.getLatestBlockhash();

      tx.recentBlockhash = blockhashWithHeight.blockhash;

      tx.feePayer = localWallet.publicKey;

      tx.sign(localWallet);

      const txSig = await connection.sendRawTransaction(tx.serialize(), {
        preflightCommitment: "finalized",
      });

      await connection.confirmTransaction(txSig, "finalized");

      console.log("Transaction signature", txSig);
    } catch (e) {
      console.error(e);
    }
  });

  it("fetch the research paper", async () => {
    try {
      const paperContentHash = "48y2ehidkhdkhadahkhadhiakhdiaydh"; //32 bytes

      const seeds = [
        Buffer.from("deres_paper"),
        Buffer.from(paperContentHash),
        localWallet.publicKey.toBuffer(),
      ];

      const [paperPda, bump] = solana.PublicKey.findProgramAddressSync(
        seeds,
        sdk.PROGRAM_ID
      );

      console.log("Paper pda", paperPda.toBase58());

      let acc_info = await connection.getAccountInfo(paperPda);

      if (!acc_info) {
        console.error("Account not found");
        return;
      }

      const [acc, _id] =
        sdk.accountProviders.ResearchPaper.fromAccountInfo(acc_info);
      console.log(acc.pretty());
    } catch (e) {
      console.error(e);
    }
  });
});
