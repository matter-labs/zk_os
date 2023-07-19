## Zk OS

Ideally: Operation system for the next "world computer", that allows to run untrusted user programs expressed in native code (RISC-V ISA in our case, 32/64 bit, I + M set). We assume strictly serial execution model, with blocking IPC, and each program would pass resources from itself to the callee (mainly time ticks).

In practice for now: example repo to work together with our `risc_v_simulator` and used to implement more restricted version of isolation.

## Grand vision

Once upon a time there was an Idea for Ethereum shards to have different interpreters, but so far it didn't happen. Let's try to fix this. Assume (via `risc_v_simulator` repo) that one has an access to ZK provable simulator of RISC-V 32 (now in the simulator) or 64 (that it most likely will be in practice) bit instruction set with I+M extensions (no usermode/MMU for now). It was oracle access to non-deterministic data via quasi-UART (read word/write word). And let's try to build an execution environment of smart-contracts that live in the common 32-byte address space, follow the same binary interface (e.g. Solidity's ABI), but their code can be either
- bytecode of our zkVM
- EVM bytecode
- WASM

and all of them can call each other! For a moment (because simulator has no interrup functionality) we can ignore resource metering, but it can actually be implemented without interrupts in our model.

So we can write a system that looks like:
- small system layer that implement IO: storage, transient storage, events, memory allocation (note - no translation, so it'll require some creativity down the road). Note that we can implement all the nice tricks that were demonstrated by zkSync Era, for example lazy storage application, provable(!) pubdata discounts, and whatever we imagine, and all of the can be implemented by one(!) copy of Rust (read - normal language) code, and still be provable(!!!)
- three interpreters listed above. Those may require to be butchered to don't abuse allocations (or use proper `Allocator` trait implementation controlled by out system layer), or even extended (more on it below). But for example for storage access they still would go and ask the system layer like "give me storage slot X for contract Y"
- One can make any assumption about the "origin" of the interaction, but it should resemble a standard smart-contract call transaction, and in general few transactions make a block.
- when one sees a "FAR CALL" (zkVM) / "CALL" (EVM) / some special host function call to other contract (WASM) it should pass the execution to another contract along with some resources

So the task we give you with this repo, example and description - try to make such a system. Be creative - because ALL the code that you write is provable, one can do interesting things. For example - when EVM bytecode is deployed, it's possible to preprocess is and e.g. create metadata of all jumpdests, or merklize, or check bytecode for some patterns, or one-pass JIT it even. Same for example for WASM - create a lookup table of block indexes -> offsets, or even JIT to native(!) RISC-V code, and if such JIT is not overly complex and somewhat "safe" - it will be huge efficiency boost. And remember - this action is just Rust code (no circuit), and done once, and proven - so it makes sense to sometimes to O(1) preprocessing on deployments for manifold saving in runtime later one

## Another side
With this repo we also start more engaged work with community and final application developers in a form of RFPs that should say "what do you want to see in the ideal blockchain execution environment".

For example, we named "transient storage" above - it's super easy to implement (and zkSync Era has it actually for free with minimal modification of the current circuits), but was drowning in the proposals for a period of years.

Or may be it would be nice to have immutable-like variables, but not mixed in the "code" and rather just stored alongside the code itself - in special constants area. So ALL contracts that have the same CODE (that is LOGIC and LAW) would literally have same bytecode hash (for ease of comparison), regardless of what constants were chosen by the deploying party.

Be creative here and leave such proposals or feature requests for the "system layer" in issues of this repo.

## What's in the repo

The repo itself is just a small example of how one can bootstrap the system, inspired by the [blog](https://osblog.stephenmarz.com/index.html) about OS development in Rust. It's not intended to be 100% correct or pretend to be anywhere like a good OS because our execution enviroment is different, for example we do not require threads/scheduling, or memory isolation (by translation) yet (but eventually we will need it!). It's a good starting/demo point to start desining an implementation of the vision above.
