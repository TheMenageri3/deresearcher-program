import * as sdk from "../src";
import fs from "fs";
import * as solana from "@solana/web3.js";
import os from "os";

const getLocalWallet = () => {
  let homeDir = os.homedir();

  const localWalletFile = fs.readFileSync(homeDir + "/.config/solana/id.json");

  let jsonParsed = Uint8Array.from(JSON.parse(localWalletFile.toString()));

  return solana.Keypair.fromSecretKey(jsonParsed);
};

const localWallet = solana.Keypair.generate();
const wallet2 = solana.Keypair.generate();

const connection = new solana.Connection("http://127.0.0.1:8899");

console.log("Airdropping... for pubkey", localWallet.publicKey.toBase58());

const [txId1, txId2] = await Promise.all([
  connection.requestAirdrop(
    localWallet.publicKey,
    10 * solana.LAMPORTS_PER_SOL
  ),
  connection.requestAirdrop(wallet2.publicKey, 10 * solana.LAMPORTS_PER_SOL),
]);

await Promise.all([
  connection.confirmTransaction(txId1, "confirmed"),
  connection.confirmTransaction(txId2, "confirmed"),
]);

describe("Integration tests", () => {
  it("Create a ReseacherProfile account", async () => {
    try {
      const seeds = [
        Buffer.from("deres_researcher_profile"),
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
            metaDataMerkleRoot:
              "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2",
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

      await connection.confirmTransaction(txSig, "confirmed");

      const seeds2 = [
        Buffer.from("deres_researcher_profile"),
        wallet2.publicKey.toBuffer(),
      ];

      const [researcherProfilePda2, bump2] =
        solana.PublicKey.findProgramAddressSync(seeds2, sdk.PROGRAM_ID);

      console.log(
        "Researcher profile pda  2",
        researcherProfilePda2.toBase58()
      );

      const ix2 = sdk.createCreateResearcherProfileInstruction(
        {
          researcherAcc: wallet2.publicKey,
          researcherProfilePdaAcc: researcherProfilePda2,
          systemProgramAcc: solana.SystemProgram.programId,
        },
        {
          createResearcherProfile: {
            name: "jill",
            pdaBump: bump2,
            metaDataMerkleRoot:
              "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2",
          },
        }
      );

      const tx2 = new solana.Transaction().add(ix2);

      const blockhashWithHeight2 = await connection.getLatestBlockhash();

      tx2.recentBlockhash = blockhashWithHeight2.blockhash;

      tx2.feePayer = wallet2.publicKey;

      tx2.sign(wallet2);

      const txSig2 = await connection.sendRawTransaction(tx2.serialize(), {
        preflightCommitment: "confirmed",
      });

      console.log("Transaction signature 2", txSig2);

      await connection.confirmTransaction(txSig2, "confirmed");

      console.log(txSig);
    } catch (e) {
      console.error(e);
    }
  });

  it("fetch the researcher profile", async () => {
    try {
      const seeds = [
        Buffer.from("deres_researcher_profile"),
        localWallet.publicKey.toBuffer(),
      ];

      const [researcherProfilePda, bump] =
        solana.PublicKey.findProgramAddressSync(seeds, sdk.PROGRAM_ID);

      let acc_info = await connection.getAccountInfo(
        researcherProfilePda,
        "confirmed"
      );

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
      const paperContentHash =
        "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2"; //32 bytes

      const seeds = [
        Buffer.from("deres_research_paper"),
        Buffer.from(paperContentHash.slice(0, 32)),
        localWallet.publicKey.toBuffer(),
      ];

      const [paperPda, bump] = solana.PublicKey.findProgramAddressSync(
        seeds,
        sdk.PROGRAM_ID
      );

      console.log("Paper pda", paperPda.toBase58());

      const researcherProfilePda = solana.PublicKey.findProgramAddressSync(
        [
          Buffer.from("deres_researcher_profile"),
          localWallet.publicKey.toBuffer(),
        ],
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
            metaDataMerkleRoot:
              "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2",
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

      await connection.confirmTransaction(txSig, "confirmed");

      console.log("Transaction signature", txSig);
    } catch (e) {
      console.error(e);
    }
  });

  it("fetch the research paper", async () => {
    try {
      const paperContentHash =
        "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2"; //32 bytes

      const seeds = [
        Buffer.from("deres_research_paper"),
        Buffer.from(paperContentHash.slice(0, 32)),
        localWallet.publicKey.toBuffer(),
      ];

      const [paperPda, bump] = solana.PublicKey.findProgramAddressSync(
        seeds,
        sdk.PROGRAM_ID
      );

      console.log("Paper pda", paperPda.toBase58());

      let acc_info = await connection.getAccountInfo(paperPda, "confirmed");

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

  it("Approve researcher to add peer review", async () => {
    try {
      const checkerWallet = getLocalWallet();

      const researcherProfilePda = solana.PublicKey.findProgramAddressSync(
        [Buffer.from("deres_researcher_profile"), wallet2.publicKey.toBuffer()],
        sdk.PROGRAM_ID
      )[0];

      let ix = sdk.createCheckAndAssignReputationInstruction(
        {
          reputationCheckerAcc: checkerWallet.publicKey,
          researcherProfilePdaAcc: researcherProfilePda,
        },
        {
          checkAndAssignReputation: {
            reputation: 100,
          },
        }
      );

      const tx = new solana.Transaction().add(ix);

      const blockhashWithHeight = await connection.getLatestBlockhash();

      tx.recentBlockhash = blockhashWithHeight.blockhash;

      tx.feePayer = checkerWallet.publicKey;

      tx.sign(checkerWallet);

      const txSig = await connection.sendRawTransaction(tx.serialize(), {
        preflightCommitment: "confirmed",
      });

      await connection.confirmTransaction(txSig, "confirmed");

      console.log("Transaction signature", txSig);
    } catch (e) {
      console.error(e);
    }
  });

  it("Add a peer review", async () => {
    try {
      const paperContentHash =
        "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2"; //32 bytes

      const seeds = [
        Buffer.from("deres_research_paper"),
        Buffer.from(paperContentHash.slice(0, 32)),
        localWallet.publicKey.toBuffer(),
      ];

      const [paperPda, bump] = solana.PublicKey.findProgramAddressSync(
        seeds,
        sdk.PROGRAM_ID
      );

      console.log("Paper pda", paperPda.toBase58());

      const researcherProfilePda = solana.PublicKey.findProgramAddressSync(
        [Buffer.from("deres_researcher_profile"), wallet2.publicKey.toBuffer()],
        sdk.PROGRAM_ID
      )[0];

      console.log("Researcher profile pda", researcherProfilePda.toBase58());

      const [peerReviewPda, bump2] = solana.PublicKey.findProgramAddressSync(
        [
          Buffer.from("deres_peer_review"),
          paperPda.toBuffer(),
          wallet2.publicKey.toBuffer(),
        ],
        sdk.PROGRAM_ID
      );

      console.log("Peer review pda", peerReviewPda.toBase58());

      const ix = sdk.createAddPeerReviewInstruction(
        {
          reviewerAcc: wallet2.publicKey,
          researcherProfilePdaAcc: researcherProfilePda,
          paperPdaAcc: paperPda,
          peerReviewPdaAcc: peerReviewPda,
          systemProgramAcc: solana.SystemProgram.programId,
        },
        {
          addPeerReview: {
            qualityOfResearch: 100,
            potentialForRealWorldUseCase: 100,
            practicalityOfResultObtained: 100,
            domainKnowledge: 100,
            metaDataMerkleRoot:
              "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2",
            pdaBump: bump2,
          },
        }
      );

      const tx = new solana.Transaction().add(ix);

      const blockhashWithHeight = await connection.getLatestBlockhash();

      tx.recentBlockhash = blockhashWithHeight.blockhash;

      tx.feePayer = wallet2.publicKey;

      tx.sign(wallet2);

      const txSig = await connection.sendRawTransaction(tx.serialize(), {
        preflightCommitment: "confirmed",
      });

      await connection.confirmTransaction(txSig, "confirmed");

      console.log("Transaction signature", txSig);
    } catch (e) {
      console.error(e);
    }
  });

  it("Publish a research paper", async () => {
    try {
      const paperContentHash =
        "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2"; //32 bytes

      const seeds = [
        Buffer.from("deres_research_paper"),
        Buffer.from(paperContentHash.slice(0, 32)),
        localWallet.publicKey.toBuffer(),
      ];

      const [paperPda, bump] = solana.PublicKey.findProgramAddressSync(
        seeds,
        sdk.PROGRAM_ID
      );

      console.log("Paper pda", paperPda.toBase58());

      const ix = sdk.createPublishPaperInstruction(
        {
          publisherAcc: localWallet.publicKey,
          paperPdaAcc: paperPda,
        },
        {
          publishPaper: {
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

      await connection.confirmTransaction(txSig, "confirmed");

      console.log("Transaction signature", txSig);
    } catch (e) {
      console.error(e);
    }
  });

  it("Mint a research paper", async () => {
    try {
      const paperContentHash =
        "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2"; //32 bytes

      const seeds = [
        Buffer.from("deres_research_paper"),
        Buffer.from(paperContentHash.slice(0, 32)),
        localWallet.publicKey.toBuffer(),
      ];

      const [paperPda, bump1] = solana.PublicKey.findProgramAddressSync(
        seeds,
        sdk.PROGRAM_ID
      );

      console.log("Paper pda", paperPda.toBase58());

      const researcherProfilePda = solana.PublicKey.findProgramAddressSync(
        [
          Buffer.from("deres_researcher_profile"),
          localWallet.publicKey.toBuffer(),
        ],
        sdk.PROGRAM_ID
      )[0];

      console.log("Researcher profile pda", researcherProfilePda.toBase58());

      const [researchMintCollectionPda, bump2] =
        solana.PublicKey.findProgramAddressSync(
          [
            Buffer.from("deres_mint_collection"),
            localWallet.publicKey.toBuffer(),
          ],
          sdk.PROGRAM_ID
        );

      console.log(
        "Research mint collection pda",
        researchMintCollectionPda.toBase58()
      );

      const ix = sdk.createMintResearchPaperInstruction(
        {
          readerAcc: localWallet.publicKey,
          researcherProfilePdaAcc: researcherProfilePda,
          researchMintCollectionPdaAcc: researchMintCollectionPda,
          paperPdaAcc: paperPda,
          feeReceiverAcc: localWallet.publicKey,
          systemProgramAcc: solana.SystemProgram.programId,
        },
        {
          mintResearchPaper: {
            metaDataMerkleRoot:
              "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2",
            pdaBump: bump2,
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

      await connection.confirmTransaction(txSig, "confirmed");

      console.log("Transaction signature", txSig);
    } catch (e) {
      console.error(e);
    }
  });

  it("fetch the minted research paper", async () => {
    try {
      const paperContentHash =
        "0a69c09f7c1eca87a0a6fb108e3aeb1929a2e4bb732a021612730325fd5875b2"; //32 bytes

      const seeds = [
        Buffer.from("deres_research_paper"),
        Buffer.from(paperContentHash.slice(0, 32)),
        localWallet.publicKey.toBuffer(),
      ];

      const [paperPda, bump] = solana.PublicKey.findProgramAddressSync(
        seeds,
        sdk.PROGRAM_ID
      );

      console.log("Paper pda", paperPda.toBase58());

      let acc_info = await connection.getAccountInfo(paperPda, "confirmed");

      if (!acc_info) {
        console.error("Account not found");
        return;
      }

      const [acc, _id] =
        sdk.accountProviders.ResearchPaper.fromAccountInfo(acc_info);
      console.log(acc.pretty());

      console.log("Minted research paper", acc.pretty());

      const researchMintCollectionPda = solana.PublicKey.findProgramAddressSync(
        [
          Buffer.from("deres_mint_collection"),
          localWallet.publicKey.toBuffer(),
        ],
        sdk.PROGRAM_ID
      )[0];

      console.log(
        "Research mint collection pda",
        researchMintCollectionPda.toBase58()
      );

      const acc_info2 = await connection.getAccountInfo(
        researchMintCollectionPda,
        "confirmed"
      );

      if (!acc_info2) {
        console.error("Account not found");
        return;
      }

      const [acc2, _id2] =
        sdk.accountProviders.ResearchMintCollection.fromAccountInfo(acc_info2);

      console.log(acc2.pretty());
    } catch (e) {
      console.error(e);
    }
  });
});
