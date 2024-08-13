import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CashApp } from "../target/types/cash_app";

describe("cash-app", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.CashApp as Program<CashApp>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  describe("cash-app", () => {
    const provider = anchor.AnchorProvider.env();
    const program = anchor.workspace.CashApp as Program<CashApp>;

    it("A to B user flow", async () => {
      const myWallet = provider.wallet as anchor.Wallet;
      const yourWallet = new anchor.web3.Keypair();

      const [myAccount] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("cash-account"), myWallet.publicKey.toBuffer()],
        program.programId
      );

      const [yourAccount] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("cash-account"), yourWallet.publicKey.toBuffer()],
        program.programId
      );

      console.log("requesting airdrop");
      const airdropTx = await provider.connection.requestAirdrop(
        yourWallet.publicKey,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(airdropTx);

      let yourBalance = await program.provider.connection.getBalance(
        yourWallet.publicKey
      );
      console.log("Your wallet balance:", yourBalance);

      const initMe = await program.methods
        .initializeAccount()
        .accounts({
          cashAccount: myAccount,
          signer: myWallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      console.log(`Use 'solana confirm -v ${initMe}' to see the logs`);

      await anchor.getProvider().connection.confirmTransaction(initMe);

      const initYou = await program.methods
        .initializeAccount()
        .accounts({
          cashAccount: yourAccount,
          signer: yourWallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([yourWallet])
        .rpc();
      console.log(`Initialized your account : ${initYou}' `);

      await anchor.getProvider().connection.confirmTransaction(initYou);
    });
  });
});
