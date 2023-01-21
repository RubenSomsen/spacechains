# Spacechains Mining Demo
### Intro
This proof of concept will show how users can compete to commit a spacechain block into the Bitcoin blockchain.

Spacechains are one-way pegged sidechains for Bitcoin. Watch [this video](https://vimeo.com/703246895/d89aba6e56), read [this article](https://medium.com/@RubenSomsen/21-million-bitcoins-to-rule-all-sidechains-the-perpetual-one-way-peg-96cb2f8ac302), and find [more links here](https://tiny.cc/somsen#spacechains).

If you get stuck, join the [spacechains Telegram chat](https://t.me/spacechains) and ask for help.

### High level overview
The software contains a set of pre-signed covenant transactions (one per block) which each contain a single freely spendable output. Any user who succeeds to spend from it using the steps below, gets to commit their spacechain block hash. RBF is enforced, meaning only the transaction with the highest fee will succeed.

## Steps

### 1. Connect to signet

You'll need Bitcoin Core. In `bitcoin.conf` set `signet=1` and synchronize your node (takes less than 15 minutes). Or, for the purposes of this demo you can just start it in a terminal windown with `bitcoind --signet` and then do all your CLI calls with `bitcoin-cli --signet`.

Create a new wallet with `-named createwallet wallet_name=spacechain-demo descriptors=false`, then generate a new address by typing `getnewaddress` and then proceed to the [signet faucet](https://signet.bc-2.jp/) to receive 0.001 coins on it.

### 2. Locate the latest pre-signed covenant transaction

Type in `importaddress 2NEcniP26o4oF2jUHgTAcncJnKt653gxrLw` ( triggers a rescan which takes a minute). This enables you to find the latest unspent output of the covenant.

Now type in `listunspent`, and you will see information about the address you just added, as well as the coins you received from the faucet. We will need the `txid` of the covenant transaction later, but keep in mind that it changes with each block (as it gets spent and recreated).

### 3. Create an unsigned raw transaction

During this step we will generate a transaction with a single input and a single output, which will be used to pay a fee. We won't sign it until later.

We will use the results from `listunspent` to make some changes to `createrawtransaction '[{"txid":"myid","vout":0}]' "[{"address":0.0009}]'` .

Use the information from your faucet coins to replace `myid` with the txid and the vout of `0` with the correct `vout` (usually `0` or `1`).

Replace `address` with the address you used to receive the faucet coins (a new address is fine too, but let's keep it simple).

Replace `0.0009` with the amount of coins that is in the input, minus the fee you'd like to pay. If your input has 0.001 and you'd like to pay a fee of 0.0001, then 0.0009 is appropriate.

Example input:
```
createrawtransaction '[{"txid":"c25adae7f0cc4c6ce783d51ffeac79dcb184b0fa32c39ebce01dbe20ce75cc84","vout":0}]' '[{"tb1qcz6za0wwah3yn3x4f0hgeugmt7lg599gtaut6e":0.0007}]'
```
Example output:
```
0200000001f472268495d3c06f48c90a7a7b122baf944dc5b334aa79fcfbdaf2e1545a2a7a0000000000ffffffff01905f010000000000160014c0b42ebdceede249c4d54bee8cf11b5fbe8a14a800000000
```
### 4. Pick a hash
This is supposed to be a valid hash for a spacechain block, but for the purpose of this demo the content doesn't matter.

You can [generate your own hex-encoded string](https://string-functions.com/string-hex.aspx) (up to 80 bytes), or simply use this one:  `68656c6c6f20776f726c64` (hello world).

### 5. Run this software

Running this software requires compiling a rust project. If you want to keep things simple, just come to the [spacechains Telegram chat](https://t.me/spacechains) so someone can run it for you.

The steps to compile it yourself are:
```
git clone https://github.com/RubenSomsen/spacechains
cargo build --release
cd target
cd release
```

Then run the software with the following parameters:
`spacechains txid spacechain_hash rawtransaction`

Where `txid` is replaced with the txid of the covenant (see `listunspent`), `spacechain_hash` is replaced with the value that you picked in step 4, and `rawtransaction` is replaced with the output you got from step 3.

Example input:
```
spacechains 5b9ca4b31dd67bd6afeb7b4bf83a3b33e0a99c13d4f00ae075a801eacdd99546 68656c6c6f20776f726c64 0200000001757a11faa0eac7b182da1e80867ad536b56f437f5e5479a7b1486a0941d90b2f0000000000ffffffff017011010000000000160014c0b42ebdceede249c4d54bee8cf11b5fbe8a14a800000000
```
Example output:
```
Covenant tx:
02000000014695d9cdea01a875e00af0d4139ca9e0333b3af84b7bebafd67bd61db3a49c5b000000006f483045022100a23565bf43fbc944f0d7626df05e2da78819b482cec0852291efe312c7cc737202205aebe0325bb84fdf1f2578e232520c9a610e17958b4e54d2ad7aac202d49094201252103df26767289da117bea582be1aa876ba426a25cbb1ba1d709877aba2d58d9717aad51b20100000002606701000000000017a914ea6eb1dd2222157a7003cf5f013515554f63a4ed87200300000000000017a9144d35011a0615c8fb2b9c025b00ce7177c80a491a8700000000
Fee-bumping cpfp tx:
0200000002757a11faa0eac7b182da1e80867ad536b56f437f5e5479a7b1486a0941d90b2f0000000000ffffffff1bfc856b8ca14703f9208c1bd4d2028526a24e19f5987c42c41e2c7b89bf411901000000040300b28b00000000027011010000000000160014c0b42ebdceede249c4d54bee8cf11b5fbe8a14a820030000000000000d6a0b68656c6c6f20776f726c6400000000
```

### Docker Alternative

Docker can be used to create reproducible builds without the need to install any other dependencies on the host.

The steps to build the docker container are:
```
git clone https://github.com/RubenSomsen/spacechains
cd spacechains
docker build -t spacechains .
```

The parameters can then be specified when running the docker container:
```
docker run spacechains 5b9ca4b31dd67bd6afeb7b4bf83a3b33e0a99c13d4f00ae075a801eacdd99546 68656c6c6f20776f726c64 0200000001757a11faa0eac7b182da1e80867ad536b56f437f5e5479a7b1486a0941d90b2f0000000000ffffffff017011010000000000160014c0b42ebdceede249c4d54bee8cf11b5fbe8a14a800000000
```

### 6. Verify and sign the cpfp tx

The cpfp tx that you received in the previous step still needs to be signed, but you have to be certain that your input and output were not altered. In order to verify, run the following command in Bitcoin Core:
`decoderawtransaction cpfp_tx`, where `cpfp_tx` is replaced with the actual output.

If everything checks out, we are ready to sign, which is done with the command `signrawtransactionwithwallet cpfp_tx` (again, replacing `cpfp_tx` with the actual output). The output that you need is the `hex` value.

Note that Bitcoin Core's output will also show you `complete": false` and `error": "Input not found or already spent`. This is expected behavior and can be ignored. The transaction is also spending an output from the covenant tx that we generated in the prior step, so Bitcoin Core is not aware of its existence yet.

Example input:
```
0200000002757a11faa0eac7b182da1e80867ad536b56f437f5e5479a7b1486a0941d90b2f0000000000ffffffff1bfc856b8ca14703f9208c1bd4d2028526a24e19f5987c42c41e2c7b89bf411901000000040300b28b00000000027011010000000000160014c0b42ebdceede249c4d54bee8cf11b5fbe8a14a820030000000000000d6a0b68656c6c6f20776f726c6400000000
```
Example output (the `hex` value of the output):
```
02000000000102757a11faa0eac7b182da1e80867ad536b56f437f5e5479a7b1486a0941d90b2f0000000000ffffffff1bfc856b8ca14703f9208c1bd4d2028526a24e19f5987c42c41e2c7b89bf411901000000040300b28b00000000027011010000000000160014c0b42ebdceede249c4d54bee8cf11b5fbe8a14a820030000000000000d6a0b68656c6c6f20776f726c640247304402205067d0eec97eed9d3e63180b0d5e44214979ac1cf849a329761e04b36170396602206474c16aa8f5d4ba2a880ed31dbdbbaf44905496744755f14bb8d555f6bc21370121037a0ce5ccc9d7a073553757824605d51dab79533d59025fac2d68be39f49da1390000000000
```
### 7. Broadcast the transactions

There are two transactions to broadcast  – the covenant tx (from step 5) and the signed transaction (from step 6). Both can be sent with the command `sendrawtransaction tx` where `tx` is replaced with the actual transaction. Note that the covenant tx MUST be sent first, as the signed transaction depends on it.

Also note that someone else could have already sent their transactions, in which case your transaction will only go through if you paid a higher fee than them. You can also test this by replacing your own transaction by making a new one that pays a higher fee.

## Success

If your transaction went through, it means you have successfully mined a spacechain block. Congratulations!

Or perhaps someone else bid more than you? In that case you'll have to try again with a higher fee.

If you had any issues come report them in the [spacechains Telegram chat](https://t.me/spacechains).

Thank you for your time and interest in my work.

My other projects can be found [here](https://tiny.cc/somsen).
