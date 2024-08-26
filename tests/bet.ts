import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bet } from "../target/types/bet";

import { waitForTransaction } from "./helpers";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import { expect } from "chai"; 

describe("bet", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Bet as Program<Bet>;

  const connection = anchor.getProvider().connection;

  const owner = anchor.AnchorProvider.env().wallet; 
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();


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
    // Variables necesarias
    const amount = new anchor.BN(1e9); // 1 SOL en lamports
    const isHeads = false; // El usuario acepta la apuesta por "tails"

    // Determinar la cuenta de la apuesta utilizando el bet_id
    /*
    const [program, _] = await publicKey.findProgramAddress(
      [Buffer.from("bet")], 
      program.programId
    );
    */

    //   creamos una apuesta
    // todo
  });
});
