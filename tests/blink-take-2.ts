import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlinkTake2 } from "../target/types/blink_take_2";
import { expect } from "chai";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("blink-take-2", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BlinkTake2 as Program<BlinkTake2>;

  let mockPythFeedPDA: PublicKey;
  let mockPythFeedBump: number;
  let marketPDA: PublicKey;
  let marketBump: number;
  let userKeypair: Keypair;
  let userPositionPDA: PublicKey;

  const TEAM_WALLET = new PublicKey("GerW59qscGWPJarbe8Px3sUVEXJ269Z9RQndYc9MWxCe");

  before(async () => {
    // Generate PDA for mock Pyth feed
    [mockPythFeedPDA, mockPythFeedBump] = await PublicKey.findProgramAddress(
      [Buffer.from("mock_pyth_feed"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    // Initialize and fund userKeypair
    userKeypair = Keypair.generate();
    const signature = await provider.connection.requestAirdrop(userKeypair.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature);

    // Initialize mock Pyth feed
    const initialPrice = new anchor.BN(50000 * 1e6); // $50,000 with 6 decimal places
    await program.methods
      .initializeMockPythFeed(initialPrice)
      .accounts({
        priceFeed: mockPythFeedPDA,
        payer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Create market
    const memecoinSymbol = "DOGE";
    const feedId = mockPythFeedPDA.toBase58();
    const duration = new anchor.BN(86400); // 1 day in seconds

    [marketPDA, marketBump] = await PublicKey.findProgramAddress(
      [Buffer.from("market"), provider.wallet.publicKey.toBuffer(), Buffer.from(memecoinSymbol)],
      program.programId
    );

    await program.methods
      .createMarket(memecoinSymbol, feedId, duration)
      .accounts({
        market: marketPDA,
        authority: provider.wallet.publicKey,
        priceFeed: mockPythFeedPDA,
        teamWallet: TEAM_WALLET,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Setup completed successfully");
  });

  it("Verifies mock Pyth feed initialization", async () => {
    const mockFeedAccount = await program.account.mockPythFeed.fetch(mockPythFeedPDA);
    expect(mockFeedAccount.price.toNumber()).to.equal(50000 * 1e6);
    console.log("Mock Pyth feed verified successfully");
  });

  it("Verifies market creation", async () => {
    const marketAccount = await program.account.market.fetch(marketPDA);
    expect(marketAccount.memecoinSymbol).to.equal("DOGE");
    expect(marketAccount.feedId).to.equal(mockPythFeedPDA.toBase58());
    expect(marketAccount.duration.toNumber()).to.equal(86400);
    expect(marketAccount.resolved).to.be.false;
    console.log("Market creation verified successfully");
  });

  it("Places bets on the market", async () => {
    console.log("Starting 'Places bets on the market' test");
    console.log("User public key:", userKeypair.publicKey.toBase58());
    console.log("Market PDA:", marketPDA.toBase58());
  
    // Verify market account before placing bets
    const marketAccount = await program.account.market.fetch(marketPDA);
    console.log("Market account:", marketAccount);
  
    // Generate PDA for user position
    [userPositionPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("user_position"), marketPDA.toBuffer(), userKeypair.publicKey.toBuffer()],
      program.programId
    );
    console.log("User position PDA:", userPositionPDA.toBase58());
  
    const betAmount = new anchor.BN(0.1 * LAMPORTS_PER_SOL); // 0.1 SOL
    const yesBet = true;
    const noBet = false;
  
    console.log("Placing 'Yes' bet...");
    try {
      await program.methods
        .placeBet(betAmount, yesBet)
        .accounts({
          market: marketPDA,
          userPosition: userPositionPDA,
          user: userKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();
      console.log("'Yes' bet placed successfully");
    } catch (error) {
      console.error("Error placing 'Yes' bet:", error);
      throw error;
    }
  
    console.log("Placing 'No' bet...");
    try {
      await program.methods
        .placeBet(betAmount, noBet)
        .accounts({
          market: marketPDA,
          userPosition: userPositionPDA,
          user: userKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();
      console.log("'No' bet placed successfully");
    } catch (error) {
      console.error("Error placing 'No' bet:", error);
      throw error;
    }
  
    // Fetch updated market and user position accounts
    const updatedMarket = await program.account.market.fetch(marketPDA);
    console.log("Updated market account:", updatedMarket);
  
    let userPosition;
    try {
      userPosition = await program.account.userPosition.fetch(userPositionPDA);
      console.log("User position account:", userPosition);
    } catch (error) {
      console.error("Error fetching user position:", error);
      throw error;
    }
  
    // Verify user position
    console.log("Verifying user position...");
    expect(userPosition.yesShares.toNumber()).to.be.greaterThan(0);
    expect(userPosition.noShares.toNumber()).to.be.greaterThan(0);
    expect(userPosition.user.toBase58()).to.equal(userKeypair.publicKey.toBase58());
    expect(userPosition.market.toBase58()).to.equal(marketPDA.toBase58());
  
    // Verify market state
    console.log("Verifying market state...");
    expect(updatedMarket.totalYesShares.toNumber()).to.equal(userPosition.yesShares.toNumber());
    expect(updatedMarket.totalNoShares.toNumber()).to.equal(userPosition.noShares.toNumber());
    expect(updatedMarket.totalFunds.toNumber()).to.equal(betAmount.toNumber() * 2);
  
  // Verify funds transfer
  const userBalance = await provider.connection.getBalance(userKeypair.publicKey);
  const totalBetAmount = betAmount.toNumber() * 2;
  const expectedBalanceUpperBound = 2 * LAMPORTS_PER_SOL - totalBetAmount;
  const expectedBalanceLowerBound = expectedBalanceUpperBound - 2000000; // Allow for up to 0.002 SOL in fees and rounding

  console.log("User balance after bets:", userBalance);
  console.log("Expected balance range:", expectedBalanceLowerBound, "to", expectedBalanceUpperBound);

  expect(userBalance).to.be.at.least(expectedBalanceLowerBound).and.at.most(expectedBalanceUpperBound);

  console.log("Bets placed successfully");
});

});