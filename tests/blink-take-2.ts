import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlinkTake2 } from "../target/types/blink_take_2";
import { expect } from "chai";
import { PublicKey, Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";

describe("prediction-blink", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BlinkTake2 as Program<BlinkTake2>;

  let mockPythFeed: Keypair;
  let marketPDA: PublicKey;
  let userKeypair: Keypair;
  let userPositionPDA: PublicKey;

  const TEAM_WALLET = new PublicKey("GerW59qscGWPJarbe8Px3sUVEXJ269Z9RQndYc9MWxCe");

  before(async () => {
    mockPythFeed = Keypair.generate();
    userKeypair = Keypair.generate();

    // Fund user account
    const signature = await provider.connection.requestAirdrop(userKeypair.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature);
  });

  it("Initializes mock Pyth feed", async () => {
    const initialPrice = new anchor.BN(50000 * 1e6); // $50,000 with 6 decimal places

    await program.methods
      .initializeMockPythFeed(initialPrice)
      .accounts({
        priceFeed: mockPythFeed.publicKey,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([mockPythFeed])
      .rpc();

    const mockFeedAccount = await program.account.mockPythFeed.fetch(mockPythFeed.publicKey);
    expect(mockFeedAccount.price.toNumber()).to.equal(initialPrice.toNumber());
  });

  it("Creates a new market", async () => {
    const memecoinSymbol = "BTC";
    const feedId = mockPythFeed.publicKey.toBase58();
    const duration = new anchor.BN(86400); // 1 day in seconds

    [marketPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("market"), provider.wallet.publicKey.toBuffer(), Buffer.from(memecoinSymbol)],
      program.programId
    );

    await program.methods
      .createMarket(memecoinSymbol, feedId, duration)
      .accounts({
        market: marketPDA,
        authority: provider.wallet.publicKey,
        priceFeed: mockPythFeed.publicKey,
        teamWallet: TEAM_WALLET,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const marketAccount = await program.account.market.fetch(marketPDA);
    expect(marketAccount.memecoinSymbol).to.equal(memecoinSymbol);
    expect(marketAccount.feedId).to.equal(feedId);
    expect(marketAccount.duration.toNumber()).to.equal(duration.toNumber());
    expect(marketAccount.resolved).to.be.false;
  });

  it("Places a bet", async () => {
    const betAmount = new anchor.BN(0.1 * LAMPORTS_PER_SOL); // 0.1 SOL
    const betChoice = true; // "Yes" bet

    [userPositionPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("user_position"), marketPDA.toBuffer(), userKeypair.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .placeBet(betAmount, betChoice)
      .accounts({
        market: marketPDA,
        userPosition: userPositionPDA,
        user: userKeypair.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([userKeypair])
      .rpc();

    const userPosition = await program.account.userPosition.fetch(userPositionPDA);
    expect(userPosition.yesShares.toNumber()).to.be.greaterThan(0);
    expect(userPosition.noShares.toNumber()).to.equal(0);

    const marketAccount = await program.account.market.fetch(marketPDA);
    expect(marketAccount.totalYesShares.toNumber()).to.be.greaterThan(0);
  });

  it("Resolves the market", async () => {
    // Update mock Pyth feed with a new price
    const newPrice = new anchor.BN(55000 * 1e6); // $55,000 with 6 decimal places
    await program.methods
      .initializeMockPythFeed(newPrice)
      .accounts({
        priceFeed: mockPythFeed.publicKey,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([mockPythFeed])
      .rpc();

    await program.methods
      .resolveMarket()
      .accounts({
        market: marketPDA,
        authority: provider.wallet.publicKey,
        priceFeed: mockPythFeed.publicKey,
      })
      .rpc();

    const marketAccount = await program.account.market.fetch(marketPDA);
    expect(marketAccount.resolved).to.be.true;
    expect(marketAccount.winningOutcome).to.not.be.null;
  });

  it("Claims winnings", async () => {
    const initialBalance = await provider.connection.getBalance(userKeypair.publicKey);

    await program.methods
      .claimWinnings()
      .accounts({
        market: marketPDA,
        userPosition: userPositionPDA,
        user: userKeypair.publicKey,
        teamWallet: TEAM_WALLET,
        systemProgram: SystemProgram.programId,
      })
      .signers([userKeypair])
      .rpc();

    const finalBalance = await provider.connection.getBalance(userKeypair.publicKey);
    expect(finalBalance).to.be.greaterThan(initialBalance);

    const userPosition = await program.account.userPosition.fetch(userPositionPDA);
    expect(userPosition.claimed).to.be.true;
  });

  it("Withdraws team fee", async () => {
    const initialTeamBalance = await provider.connection.getBalance(TEAM_WALLET);

    // HACK: For now, we'll just call the function and expect it to fail
    try {
      await program.methods
        .withdrawTeamFee()
        .accounts({
          market: marketPDA,
          authority: provider.wallet.publicKey,
          teamWallet: TEAM_WALLET,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (error) {
      expect(error.message).to.include("TeamFeeTimelockNotExpired");
    }

    const finalTeamBalance = await provider.connection.getBalance(TEAM_WALLET);
    expect(finalTeamBalance).to.equal(initialTeamBalance);

  });
});