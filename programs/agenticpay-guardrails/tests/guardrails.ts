import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { assert, expect } from "chai";

describe("agenticpay-guardrails", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AgenticpayGuardrails as Program<any>;
  const authority = provider.wallet;

  const agentKeypair = Keypair.generate();
  const sessionKeypair = Keypair.generate();
  const recipientKeypair = Keypair.generate();

  let guardrailsPda: PublicKey;
  let guardrailsBump: number;

  let sessionKeyPda: PublicKey;
  let sessionKeyBump: number;

  const DAILY_SPEND_LIMIT = new anchor.BN(10_000_000); // 0.01 SOL (10M lamports)
  const MAX_TX_LIMIT = new anchor.BN(2_000_000);        // 0.002 SOL (2M lamports)
  const OVERALL_SPEND_CAP = new anchor.BN(100_000_000); // 0.1 SOL (100M lamports)
  const SESSION_ALLOWANCE = new anchor.BN(5_000_000);   // 0.005 SOL

  before(async () => {
    // Derive Guardrails PDA: [b"guardrails", agent.pubkey]
    [guardrailsPda, guardrailsBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("guardrails"), agentKeypair.publicKey.toBuffer()],
      program.programId
    );

    // Derive SessionKey PDA: [b"session_key", guardrailsPda, sessionKeypair.pubkey]
    [sessionKeyPda, sessionKeyBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("session_key"),
        guardrailsPda.toBuffer(),
        sessionKeypair.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Airdrop SOL to agent wallet
    const airdropSig = await provider.connection.requestAirdrop(
      agentKeypair.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: airdropSig,
      ...latestBlockhash,
    });
  });

  it("Initializes spending guardrails for an agent wallet", async () => {
    await program.methods
      .initializeGuardrails(DAILY_SPEND_LIMIT, MAX_TX_LIMIT, OVERALL_SPEND_CAP)
      .accounts({
        guardrails: guardrailsPda,
        agent: agentKeypair.publicKey,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const account = await program.account.guardrailAccount.fetch(guardrailsPda);
    assert.ok(account.authority.equals(authority.publicKey));
    assert.ok(account.agent.equals(agentKeypair.publicKey));
    assert.equal(account.dailySpendLimit.toString(), DAILY_SPEND_LIMIT.toString());
    assert.equal(account.maxTxLimit.toString(), MAX_TX_LIMIT.toString());
    assert.equal(account.overallSpendCap.toString(), OVERALL_SPEND_CAP.toString());
    assert.equal(account.currentDailySpend.toString(), "0");
    assert.equal(account.currentTotalSpend.toString(), "0");
    assert.isFalse(account.isPaused);
  });

  it("Validates and records a legitimate payment within limits", async () => {
    const paymentAmount = new anchor.BN(1_000_000); // 1M lamports (<= 2M limit)

    await program.methods
      .validateAndRecordPayment(paymentAmount)
      .accounts({
        guardrails: guardrailsPda,
        agent: agentKeypair.publicKey,
      })
      .signers([agentKeypair])
      .rpc();

    const account = await program.account.guardrailAccount.fetch(guardrailsPda);
    assert.equal(account.currentDailySpend.toString(), "1000000");
    assert.equal(account.currentTotalSpend.toString(), "1000000");
  });

  it("Rejects payment exceeding max per-transaction limit", async () => {
    const excessiveAmount = new anchor.BN(3_000_000); // 3M lamports (> 2M max tx)

    try {
      await program.methods
        .validateAndRecordPayment(excessiveAmount)
        .accounts({
          guardrails: guardrailsPda,
          agent: agentKeypair.publicKey,
        })
        .signers([agentKeypair])
        .rpc();
      assert.fail("Should have thrown MaxTxLimitExceeded");
    } catch (err: any) {
      expect(err.error?.errorCode?.code || err.toString()).to.include("MaxTxLimitExceeded");
    }
  });

  it("Registers a bounded session key for deferred voucher micropayments", async () => {
    const now = Math.floor(Date.now() / 1000);
    const validUntil = new anchor.BN(now + 3600); // Valid for 1 hour

    await program.methods
      .registerSessionKey(sessionKeypair.publicKey, SESSION_ALLOWANCE, validUntil)
      .accounts({
        guardrails: guardrailsPda,
        sessionAccount: sessionKeyPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const sessionAccount = await program.account.sessionKeyAccount.fetch(sessionKeyPda);
    assert.ok(sessionAccount.authority.equals(authority.publicKey));
    assert.ok(sessionAccount.guardrails.equals(guardrailsPda));
    assert.ok(sessionAccount.sessionKey.equals(sessionKeypair.publicKey));
    assert.equal(sessionAccount.allowance.toString(), SESSION_ALLOWANCE.toString());
    assert.equal(sessionAccount.spent.toString(), "0");
    assert.isFalse(sessionAccount.isRevoked);
  });

  it("Settles a valid deferred voucher within session allowance", async () => {
    const voucherAmount = new anchor.BN(500_000); // 0.5M lamports
    const voucherNonce = new anchor.BN(1001);

    await program.methods
      .settleVoucher(voucherAmount, voucherNonce)
      .accounts({
        guardrails: guardrailsPda,
        sessionAccount: sessionKeyPda,
        settler: authority.publicKey,
      })
      .rpc();

    const sessionAccount = await program.account.sessionKeyAccount.fetch(sessionKeyPda);
    assert.equal(sessionAccount.spent.toString(), "500000");

    const guardrailsAccount = await program.account.guardrailAccount.fetch(guardrailsPda);
    assert.equal(guardrailsAccount.currentDailySpend.toString(), "1500000");
  });

  it("Rejects voucher settling beyond session key allowance", async () => {
    const excessiveVoucher = new anchor.BN(5_000_000); // Exceeds remaining 4.5M allowance
    const voucherNonce = new anchor.BN(1002);

    try {
      await program.methods
        .settleVoucher(excessiveVoucher, voucherNonce)
        .accounts({
          guardrails: guardrailsPda,
          sessionAccount: sessionKeyPda,
          settler: authority.publicKey,
        })
        .rpc();
      assert.fail("Should have thrown SessionAllowanceExceeded");
    } catch (err: any) {
      expect(err.error?.errorCode?.code || err.toString()).to.include("SessionAllowanceExceeded");
    }
  });

  it("Activates emergency pause and halts all payment transactions", async () => {
    // Activate emergency circuit breaker
    await program.methods
      .pause()
      .accounts({
        guardrails: guardrailsPda,
        authority: authority.publicKey,
      })
      .rpc();

    const account = await program.account.guardrailAccount.fetch(guardrailsPda);
    assert.isTrue(account.isPaused);

    // Attempt payment while paused
    try {
      await program.methods
        .validateAndRecordPayment(new anchor.BN(100_000))
        .accounts({
          guardrails: guardrailsPda,
          agent: agentKeypair.publicKey,
        })
        .signers([agentKeypair])
        .rpc();
      assert.fail("Should have thrown ProgramPaused");
    } catch (err: any) {
      expect(err.error?.errorCode?.code || err.toString()).to.include("ProgramPaused");
    }

    // Unpause for future operations
    await program.methods
      .unpause()
      .accounts({
        guardrails: guardrailsPda,
        authority: authority.publicKey,
      })
      .rpc();

    const unpausedAccount = await program.account.guardrailAccount.fetch(guardrailsPda);
    assert.isFalse(unpausedAccount.isPaused);
  });
});
