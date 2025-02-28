import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("copy-trading", () => {
  // Use the provided wallet and connection.
  const provider = anchor.getProvider();
  anchor.setProvider(provider);

  const program = anchor.workspace.CopyTrading as Program<any>;

  // Use the provided wallet for all actions.
  const authorityWallet = provider.wallet;
  const masterTraderWallet = provider.wallet;
  const followerWallet = provider.wallet;

  // Generate a keypair for the copy trading state account.
  const copyTradingAccount = anchor.web3.Keypair.generate();

  // Define variables for later use.
  let masterTraderAccount: anchor.web3.PublicKey;
  let mintAddress: anchor.web3.PublicKey;
  let followerTokenAccount: anchor.web3.PublicKey;
  let vaultTokenAccount: anchor.web3.PublicKey;
  let vaultPda: anchor.web3.PublicKey;
  let vaultBump: number;
  let followerPda: anchor.web3.PublicKey;
  let followerBump: number;

  // Test parameters.
  const masterTraderName = "Top Trader";
  const masterTraderDesc = "Expert in Solana DeFi";
  const depositAmount = new anchor.BN(100_000_000); // 100 tokens with 6 decimals

  before(async () => {
    console.log("Starting setup for copy trading tests...");
    console.log("----------------------------------------------------");

    console.log(
      "Using provided funded wallet:",
      authorityWallet.publicKey.toString()
    );

    try {
      // Create a SPL token mint for testing.
      mintAddress = await createMint(
        provider.connection,
        provider.wallet.payer,
        provider.wallet.publicKey,
        null,
        6
      );
      console.log("Created test token mint:", mintAddress.toString());

      // Create a token account for the follower.
      followerTokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        provider.wallet.payer,
        mintAddress,
        followerWallet.publicKey
      );
      console.log(
        "Created follower token account:",
        followerTokenAccount.toString()
      );

      // Mint tokens to the follower's token account.
      await mintTo(
        provider.connection,
        provider.wallet.payer,
        mintAddress,
        followerTokenAccount,
        provider.wallet.payer,
        500_000_000 // 500 tokens
      );
      console.log("Minted 500 tokens to follower");
    } catch (err) {
      console.error("Error during setup:", err);
      throw err;
    }

    console.log("Setup complete!");
    console.log("----------------------------------------------------");
  });

  it("Initializes the copy trading program", async () => {
    console.log("Testing program initialization...");

    try {
      await program.methods
        .initialize()
        .accounts({
          copyTrading: copyTradingAccount.publicKey,
          authority: authorityWallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([copyTradingAccount])
        .rpc();

      const account = await program.account.copyTrading.fetch(
        copyTradingAccount.publicKey
      );

      console.log(
        "Program initialized with authority:",
        account.authority.toString()
      );
      console.log("Master trader count:", account.masterTraderCount.toString());

      if (
        account.authority.toString() !== authorityWallet.publicKey.toString()
      ) {
        throw new Error("Authority should match the provided wallet");
      } else {
        console.log("Authority matches:", account.authority.toString());
      }

      if (account.masterTraderCount.toString() !== "0") {
        throw new Error("Master trader count should be 0");
      } else {
        console.log(
          "Master trader count is correct:",
          account.masterTraderCount.toString()
        );
      }

      console.log("Initialization test passed!");
    } catch (err) {
      console.error("Error during initialization:", err);
      throw err;
    }
  });

  it("Registers a master trader", async () => {
    console.log("Testing master trader registration...");

    const masterTraderAccountKeypair = anchor.web3.Keypair.generate();
    masterTraderAccount = masterTraderAccountKeypair.publicKey;

    await program.methods
      .registerMasterTrader(masterTraderName, masterTraderDesc)
      .accounts({
        copyTrading: copyTradingAccount.publicKey,
        masterTrader: masterTraderAccountKeypair.publicKey,
        authority: masterTraderWallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([masterTraderAccountKeypair])
      .rpc();

    const copyTradingState = await program.account.copyTrading.fetch(
      copyTradingAccount.publicKey
    );
    const masterTraderState = await program.account.masterTrader.fetch(
      masterTraderAccountKeypair.publicKey
    );

    console.log("Master trader registered with name:", masterTraderState.name);
    console.log("Master trader description:", masterTraderState.description);
    console.log(
      "Master trader authority:",
      masterTraderState.authority.toString()
    );
    console.log(
      "Updated master trader count:",
      copyTradingState.masterTraderCount.toString()
    );

    if (masterTraderState.name !== masterTraderName) {
      throw new Error("Master trader name should match");
    } else {
      console.log("Master trader name is correct:", masterTraderState.name);
    }

    if (masterTraderState.description !== masterTraderDesc) {
      throw new Error("Master trader description should match");
    } else {
      console.log(
        "Master trader description is correct:",
        masterTraderState.description
      );
    }

    if (
      masterTraderState.authority.toString() !==
      masterTraderWallet.publicKey.toString()
    ) {
      throw new Error(
        "Master trader authority should match the provided wallet"
      );
    } else {
      console.log(
        "Master trader authority is correct:",
        masterTraderState.authority.toString()
      );
    }

    if (masterTraderState.totalFollowers.toString() !== "0") {
      throw new Error("Total followers should be 0");
    } else {
      console.log(
        "Total followers is correct:",
        masterTraderState.totalFollowers.toString()
      );
    }

    if (copyTradingState.masterTraderCount.toString() !== "1") {
      throw new Error("Master trader count should be 1");
    } else {
      console.log(
        "Master trader count is correct:",
        copyTradingState.masterTraderCount.toString()
      );
    }

    console.log("Master trader registration test passed!");
  });

  it("Allows a user to follow a trader", async () => {
    console.log("Testing follower functionality...");

    // Derive the PDAs for the follower and vault.
    [followerPda, followerBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("follower"),
          followerWallet.publicKey.toBuffer(),
          masterTraderAccount.toBuffer(),
        ],
        program.programId
      );

    [vaultPda, vaultBump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault"),
        followerWallet.publicKey.toBuffer(),
        masterTraderAccount.toBuffer(),
      ],
      program.programId
    );

    console.log("Generated follower PDA:", followerPda.toString());
    console.log("Generated vault PDA:", vaultPda.toString());

    // Use getOrCreateAssociatedTokenAccount to ensure the vault token account is valid.
    const vaultAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mintAddress,
      vaultPda, // PDA as the owner
      true // allowOwnerOffCurve
    );
    vaultTokenAccount = vaultAta.address;
    console.log("Vault token account address:", vaultTokenAccount.toString());

    try {
      await program.methods
        .followTrader(depositAmount)
        .accounts({
          masterTrader: masterTraderAccount,
          follower: followerPda,
          vault: vaultPda,
          userTokenAccount: followerTokenAccount,
          vaultTokenAccount: vaultTokenAccount,
          user: followerWallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .rpc();

      console.log("Follow trader instruction executed successfully");
    } catch (err) {
      console.error("Error during followTrader instruction:", err);
      throw err;
    }

    const masterTraderState = await program.account.masterTrader.fetch(
      masterTraderAccount
    );
    const followerState = await program.account.follower.fetch(followerPda);

    console.log(
      "Follower deposited amount:",
      followerState.depositedAmount.toString()
    );
    console.log(
      "Master trader total followers:",
      masterTraderState.totalFollowers.toString()
    );
    console.log("Master trader AUM:", masterTraderState.totalAum.toString());

    if (followerState.user.toString() !== followerWallet.publicKey.toString()) {
      throw new Error("Follower user should match the provided wallet");
    } else {
      console.log("Follower user matches:", followerState.user.toString());
    }

    if (
      followerState.masterTrader.toString() !== masterTraderAccount.toString()
    ) {
      throw new Error("Follower master trader reference should match");
    } else {
      console.log(
        "Follower master trader reference is correct:",
        followerState.masterTrader.toString()
      );
    }

    if (followerState.depositedAmount.toString() !== depositAmount.toString()) {
      throw new Error("Deposited amount should match");
    } else {
      console.log(
        "Deposited amount is correct:",
        followerState.depositedAmount.toString()
      );
    }

    if (!followerState.active) {
      throw new Error("Follower should be active");
    } else {
      console.log("Follower is active");
    }

    if (masterTraderState.totalFollowers.toString() !== "1") {
      throw new Error("Master trader should have 1 follower");
    } else {
      console.log(
        "Master trader follower count is correct:",
        masterTraderState.totalFollowers.toString()
      );
    }

    if (masterTraderState.totalAum.toString() !== depositAmount.toString()) {
      throw new Error("Master trader AUM should match deposit amount");
    } else {
      console.log(
        "Master trader AUM is correct:",
        masterTraderState.totalAum.toString()
      );
    }

    console.log("Follow trader test passed!");
  });
});
