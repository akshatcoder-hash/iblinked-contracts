import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlinkTake2 } from "../target/types/blink_take_2";
import { expect } from "chai";
import { PublicKey } from "@solana/web3.js";

describe("blink-take-2", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BlinkTake2 as Program<BlinkTake2>;

  let mockPythFeedPDA: PublicKey;
  let mockPythFeedBump: number;

  before(async () => {
    // Generate PDA for mock Pyth feed
    [mockPythFeedPDA, mockPythFeedBump] = await PublicKey.findProgramAddress(
      [Buffer.from("mock_pyth_feed"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Initializes mock Pyth feed", async () => {
    const initialPrice = new anchor.BN(50000 * 1e6); // $50,000 with 6 decimal places

    try {
      await program.methods
        .initializeMockPythFeed(initialPrice)
        .accounts({
          priceFeed: mockPythFeedPDA,
          payer: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const mockFeedAccount = await program.account.mockPythFeed.fetch(mockPythFeedPDA);
      expect(mockFeedAccount.price.toNumber()).to.equal(initialPrice.toNumber());
      
      console.log("Mock Pyth feed initialized successfully");
    } catch (error) {
      console.error("Error during mock Pyth feed initialization:", error);
      throw error;
    }
  });
});