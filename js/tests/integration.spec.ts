import * as sdk from "../src/generated";
import fs from "fs";
import * as solana from "@solana/web3.js";
import os from "os";

let homeDir = os.homedir();

const localWalletFile = fs.readFileSync(homeDir + "/.config/solana/id.json");

let jsonParsed = Uint8Array.from(JSON.parse(localWalletFile.toString()));

let localWallet = solana.Keypair.fromSecretKey(jsonParsed);

describe("Integration tests", () => {
  const connection = new solana.Connection("http://127.0.0.1:8899");
  //   it("Create a ReseacherProfile account", async () => {
  //     try {
  //       const seeds = [
  //         Buffer.from("deres_profile"),
  //         localWallet.publicKey.toBuffer(),
  //       ];

  //       const [researcherProfilePda, bump] =
  //         solana.PublicKey.findProgramAddressSync(seeds, sdk.PROGRAM_ID);

  //       console.log("Researcher profile pda", researcherProfilePda.toBase58());

  //       const ix = sdk.createCreateResearcherProfileInstruction(
  //         {
  //           researcherAcc: localWallet.publicKey,
  //           researcherProfilePdaAcc: researcherProfilePda,
  //           systemProgramAcc: solana.SystemProgram.programId,
  //         },
  //         {
  //           createResearcherProfile: {
  //             name: "jack",
  //           },
  //         }
  //       );

  //       const tx = new solana.Transaction().add(ix);

  //       const blockhashWithHeight = await connection.getLatestBlockhash();

  //       tx.recentBlockhash = blockhashWithHeight.blockhash;

  //       tx.feePayer = localWallet.publicKey;

  //       tx.sign(localWallet);

  //       const txSig = await connection.sendRawTransaction(tx.serialize());

  //       console.log(txSig);
  //     } catch (e) {
  //       console.error(e);
  //     }
  //   });

  it("fetch the researcher profile", async () => {
    try {
      const seeds = [
        Buffer.from("deres_profile"),
        localWallet.publicKey.toBuffer(),
      ];

      const [researcherProfilePda, bump] =
        solana.PublicKey.findProgramAddressSync(seeds, sdk.PROGRAM_ID);

      const acc =
        await sdk.accountProviders.ResearcherProfile.fromAccountAddress(
          connection,
          researcherProfilePda
        );
      console.log(acc);
    } catch (e) {
      console.error(e);
    }
  });

  it("Create a Researcher paper", async () => {
    try {
      const paperContentHash =
        "0x123456789009876543211234567890098765432123456789009876543211234"; //32 bytes

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

      const ix = sdk.createCreateResearchePaperInstruction(
        {
          publisherAcc: localWallet.publicKey,
          researcherProfilePdaAcc: researcherProfilePda,
          paperPdaAcc: paperPda,
          systemProgramAcc: solana.SystemProgram.programId,
        },
        {
          createResearchePaper: {
            paperContentHash: Array.from(paperContentHash).map((x) =>
              Number(x)
            ),
            accessFee: 100,
            metaDataMerkleRoot: [],
          },
        }
      );

      const tx = new solana.Transaction().add(ix);

      const blockhashWithHeight = await connection.getLatestBlockhash();

      tx.recentBlockhash = blockhashWithHeight.blockhash;

      tx.feePayer = localWallet.publicKey;

      tx.sign(localWallet);

      const txSig = await connection.sendRawTransaction(tx.serialize());

      console.log(txSig);
    } catch (e) {
      console.error(e);
    }
  });
});
