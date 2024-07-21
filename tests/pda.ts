import { AnchorProvider, Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

import { BlinkTake2 } from "../target/types/blink_take_2";

export class PDAHelper {
  provider: AnchorProvider;
  program: Program<BlinkTake2>;

  constructor(provider: AnchorProvider, program: Program<BlinkTake2>) {
    this.provider = provider;
    this.program = program;
  }

  priceFeedConfig() {
    let [pda, _] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("price_feed_config"),
        this.provider.wallet.publicKey.toBuffer(),
      ],
      this.program.programId
    );
    return pda;
  }

  market(coinSymbol: string) {
    let [pda, _] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("market"),
        this.provider.wallet.publicKey.toBuffer(),
        Buffer.from(coinSymbol),
      ],
      this.program.programId
    );
    return pda;
  }

  userPosition(marketPDA: PublicKey, user: PublicKey) {
    let [pda, _] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_position"), marketPDA.toBuffer(), user.toBuffer()],
      this.program.programId
    );
    return pda;
  }
}
