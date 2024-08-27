import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bet } from "../target/types/bet";

import { waitForTransaction } from "./helpers";

import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai"; 

import { Keypair, LAMPORTS_PER_SOL, sendAndConfirmTransaction, Transaction } from "@solana/web3.js";
import { BN } from "bn.js";
describe("bet", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Bet as Program<Bet>;
  const connection = anchor.getProvider().connection;
  const owner = anchor.AnchorProvider.env().wallet; 
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();
  const fee_receiver = anchor.web3.Keypair.generate();

  it("Bet initialized!", async () => { 
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
    await waitForTransaction(connection, tx );

    const betAccount = PublicKey.findProgramAddressSync(
      [Buffer.from("bet")], program.programId
    )[0];

    const betData = await program.account.bet.fetch(betAccount)
    expect(betData.amount).equal(0)
    expect(betData.tails.equals(owner.publicKey))
    expect(betData.heads.equals(fee_receiver.publicKey))
    console.log("Program account data: ", betData)
  });


  it("Bet created!", async () => {
    const seed = new anchor.BN(1); // Ejemplo de seed
    const amount = new anchor.BN(1e9); // 1 SOL en lamports (1 SOL = 1_000_000 lamports)
    const resolver = anchor.web3.Keypair.generate(); // Clave pública del resolver
    const isHeads = true; // El usuario está apostando por "heads"
    // Add your test here.
    const tx = await program.methods.createBet(
      seed, amount, resolver.publicKey, isHeads
    ).rpc(); // Faltan pasar parametros
    console.log("Your transaction signature", tx);
    await waitForTransaction(connection, tx );

    const betAccount = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("bet")], program.programId
    )[0]

    const betData = await program.account.bet.fetch(betAccount) 
    expect(betData.resolver.equals(owner.publicKey)) 
  });

  it("Bet accepted!", async () => { 
  });

  it("Bet resolved!", async () => { 
  });

  it("Bet canceled!", async () => { 
  });

  it("Bet closed!", async () => { 
    /*
    const tx = await program.methods
    .close()
    .accountsPartial({
      user: provider.wallet.publicKey,
      vaultState,
      vault,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

    console.log("\nYour transaction signature", tx);
    console.log("Your vault info", (await provider.connection.getAccountInfo(vault)));
  */
  });
});
