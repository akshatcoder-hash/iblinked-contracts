import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import crypto from "node:crypto";
import { suite, test, expect, assert } from "vitest";

import { PDAHelper } from "./pda";
import { TEAM_WALLET } from "./constants";
import { BlinkTake2 } from "../target/types/blink_take_2";
import { calculateShares } from "./utils";

suite("blink-take-2", () => {
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.BlinkTake2 as Program<BlinkTake2>;
  const pdaHelper = new PDAHelper(provider, program);

  anchor.setProvider(provider);

  const connection = provider.connection;
  const authority = provider.publicKey;
  const user = Keypair.generate();
  const feed = new PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix");

  let priceFeedConfigPDA: PublicKey;
  let marketPDA: PublicKey;
  let userPositionPDA: PublicKey;

  test("airdrop SOL to user", async () => {
    const signature = await connection.requestAirdrop(
      user.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(signature);

    const balance = await connection.getBalance(user.publicKey);
    expect(balance).toBe(2 * LAMPORTS_PER_SOL);
  });

  test("initialize price feed with authorized wallet", async () => {
    priceFeedConfigPDA = pdaHelper.priceFeedConfig(feed);

    try {
      await program.methods
        .initializePriceFeed(feed)
        .accounts({
          payer: authority,
          priceFeedConfig: priceFeedConfigPDA,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const priceFeedConfigAccountData =
        await program.account.priceFeedConfig.fetch(priceFeedConfigPDA);
      expect(priceFeedConfigAccountData.priceFeed.toString()).toBe(
        feed.toString()
      );
    } catch (err) {
      console.log(err);
      assert.fail("unexpected error");
    }
  });

  test("initialize price feed with unauthorized wallet", async () => {
    priceFeedConfigPDA = pdaHelper.priceFeedConfig(feed);

    try {
      await program.methods
        .initializePriceFeed(feed)
        .accounts({
          payer: user.publicKey,
          priceFeedConfig: priceFeedConfigPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();
    } catch (err) {
      if (err instanceof AnchorError) {
        if (err.error.errorCode.number === 2006) {
          assert.ok("test failed as expected");
          return;
        }

        console.log(err);
        assert.fail("unexpected anchor error");
      }

      console.log(err);
      assert.fail("unexpected error");
    }

    assert.fail("expected test to fail as unauthorized creator was used");
  });

  test("create market with authorized creator", async () => {
    const memeCoinSymbol = crypto.randomBytes(2).toString("hex");
    const duration = new anchor.BN(10);

    marketPDA = pdaHelper.market(memeCoinSymbol);

    try {
      await program.methods
        .createMarket(memeCoinSymbol, priceFeedConfigPDA.toBase58(), duration)
        .accounts({
          authority,
          market: marketPDA,
          priceFeedConfig: priceFeedConfigPDA,
          priceFeed: feed,
          teamWallet: TEAM_WALLET,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const marketAccountData = await program.account.market.fetch(marketPDA);
      expect(marketAccountData.memecoinSymbol).toBe(memeCoinSymbol);
      expect(marketAccountData.feedId.toString()).toBe(
        priceFeedConfigPDA.toString()
      );
      expect(marketAccountData.authority.toString()).toBe(authority.toString());
      expect(marketAccountData.duration.toNumber()).toBe(duration.toNumber());
      expect(marketAccountData.resolved).toBe(false);
    } catch (err) {
      console.log(err);
      assert.fail("unexpected error");
    }
  });

  test("create market with unauthorized creator", async () => {
    const memeCoinSymbol = crypto.randomBytes(2).toString("hex");
    const duration = new anchor.BN(10);

    try {
      await program.methods
        .createMarket(memeCoinSymbol, priceFeedConfigPDA.toBase58(), duration)
        .accounts({
          authority: user.publicKey,
          market: pdaHelper.market(memeCoinSymbol),
          priceFeedConfig: priceFeedConfigPDA,
          priceFeed: feed,
          teamWallet: TEAM_WALLET,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();
    } catch (err) {
      if (err instanceof AnchorError) {
        if (err.error.errorCode.number === 2006) {
          assert.ok("test failed as expected");
          return;
        }

        console.log(err);
        assert.fail("unexpected anchor error");
      }

      console.log(err);
      assert.fail("unexpected error");
    }

    assert.fail("expected test to fail as unauthorized creator is used");
  });

  test("create user for an existing market", async () => {
    userPositionPDA = pdaHelper.userPosition(marketPDA, user.publicKey);

    try {
      await program.methods
        .createUser()
        .accounts({
          market: marketPDA,
          user: user.publicKey,
          userPosition: userPositionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const userPositionAccountData = await program.account.userPosition.fetch(
        userPositionPDA
      );

      expect(userPositionAccountData.claimed).toBe(false);
      expect(userPositionAccountData.market.toString()).toBe(
        marketPDA.toString()
      );
      expect(userPositionAccountData.noShares.toNumber()).toBe(0);
      expect(userPositionAccountData.yesShares.toNumber()).toBe(0);
      expect(userPositionAccountData.user.toString()).toBe(
        user.publicKey.toString()
      );
    } catch (err) {
      console.log(err);
      assert.fail("unexpected error");
    }
  });

  test("create an user for non-existing market", async () => {
    const marketPDA = pdaHelper.market(crypto.randomBytes(2).toString("hex"));
    const userPositionPDA = pdaHelper.userPosition(marketPDA, user.publicKey);

    try {
      await program.methods
        .createUser()
        .accounts({
          market: marketPDA,
          user: user.publicKey,
          userPosition: userPositionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();
    } catch (err) {
      if (err instanceof AnchorError) {
        if (err.error.errorCode.number === 3012) {
          assert.ok("test failed as expected");
          return;
        }

        console.log(err);
        assert.fail("unexpected anchor error");
      }

      console.log(err);
      assert.fail("unexpected error");
    }

    assert.fail("expected test to fail as the market wasn't yet created");
  });

  test("place a bet", async () => {
    const amount = new anchor.BN(0.1 * LAMPORTS_PER_SOL);
    const yesChoice = true;
    const noChoice = false;

    try {
      await program.methods
        .placeBet(amount, yesChoice)
        .accounts({
          market: marketPDA,
          user: user.publicKey,
          userPosition: userPositionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      await program.methods
        .placeBet(amount, noChoice)
        .accounts({
          market: marketPDA,
          user: user.publicKey,
          userPosition: userPositionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const userPositionAccountData = await program.account.userPosition.fetch(
        userPositionPDA
      );

      expect(userPositionAccountData.claimed).toBe(false);
      expect(userPositionAccountData.market.toString()).toBe(
        marketPDA.toString()
      );
      expect(userPositionAccountData.noShares.toNumber()).toBe(
        calculateShares(amount.toNumber())
      );
      expect(userPositionAccountData.yesShares.toNumber()).toBe(
        calculateShares(amount.toNumber())
      );
      expect(userPositionAccountData.user.toString()).toBe(
        user.publicKey.toString()
      );
    } catch (err) {
      console.log(err);
      assert.fail("unexpected error");
    }
  });

  test("place a bet lower than 90k lamports", async () => {
    const amount = new anchor.BN(90_000);
    const yesChoice = true;

    try {
      await program.methods
        .placeBet(amount, yesChoice)
        .accounts({
          market: marketPDA,
          user: user.publicKey,
          userPosition: userPositionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();
    } catch (err) {
      if (err instanceof AnchorError) {
        if (err.error.errorMessage === "Bet amount is too low") {
          assert.ok("test failed as expected");
          return;
        }

        console.log(err);
        assert.fail("unexpected anchor error");
      }

      console.log(err);
      assert.fail("unexpected error");
    }

    assert.fail("test should have failed as the bet amount is too low");
  });

  test("place a bet with insufficient lamports", async () => {
    const amount = new anchor.BN(10 * LAMPORTS_PER_SOL);
    const yesChoice = true;

    try {
      await program.methods
        .placeBet(amount, yesChoice)
        .accounts({
          market: marketPDA,
          user: user.publicKey,
          userPosition: userPositionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();
    } catch (err) {
      if (err instanceof AnchorError) {
        if (err.error.errorMessage === "Insufficient user funds") {
          assert.ok("test failed as expected");
          return;
        }

        console.log(err);
        assert.fail("unexpected anchor error");
      }

      console.log(err);
      assert.fail("unexpected error");
    }

    assert.fail(
      "test should have failed as the user has insufficient funds to bet"
    );
  });

  // NOTE: test wouldn't work, refer to the comment in cancel_bet.rs file
  test.skip("cancel bet", async () => {
    try {
      await program.methods
        .cancelBet()
        .accounts({
          market: marketPDA,
          user: user.publicKey,
          userPosition: userPositionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const userPositionAccountData = await program.account.userPosition.fetch(
        userPositionPDA
      );

      expect(userPositionAccountData.claimed).toBe(false);
      expect(userPositionAccountData.market.toString()).toBe(
        marketPDA.toString()
      );
      expect(userPositionAccountData.noShares.toNumber()).toBe(0);
      expect(userPositionAccountData.yesShares.toNumber()).toBe(0);
      expect(userPositionAccountData.user.toString()).toBe(
        user.publicKey.toString()
      );
    } catch (err) {
      console.log(err);
      assert.fail("unexpected error");
    }
  });

  test("resolve market", async () => {
    try {
      await program.methods
        .resolveMarket()
        .accounts({
          authority,
          market: marketPDA,
          priceFeedConfig: priceFeedConfigPDA,
          priceFeed: feed,
        })
        .rpc();

      const marketAccountData = await program.account.market.fetch(marketPDA);
      expect(marketAccountData.resolved).toBe(true);
      expect(marketAccountData.winningOutcome).not.toBeNull;
    } catch (err) {
      console.log(err);
      assert.fail("unexpected error");
    }
  });

  test("claim winnings", async () => {
    try {
      await program.methods
        .claimWinnings()
        .accounts({
          user: user.publicKey,
          userPosition: userPositionPDA,
          market: marketPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const userPositionAccountData = await program.account.userPosition.fetch(
        userPositionPDA
      );
      expect(userPositionAccountData.claimed).toBe(true);
      expect(userPositionAccountData.yesShares.toNumber()).toBe(0);
      expect(userPositionAccountData.noShares.toNumber()).toBe(0);
    } catch (err) {
      console.log(err);
      assert.fail("unexpected error");
    }
  });

  test.skip("withdraw team fees");
});
