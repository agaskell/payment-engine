# Payment Engine

Hey, I'm Andy. Thanks for reading my code.

## Assumptions

Some of these assumptions are probably incorrect for a real-life payment engine. I don't know. I think they're reasonable for this exercise.

- Transactions may not be disputed more than once.
- A failed withdrawal (tried to withdraw more than the available amount) may not be disputed.
- **Locked accounts do not process transactions** including disputes, resolves, or chargebacks.
- Not every row in the input file will be formatted correctly. Some rows may be formatted correctly but have incorrect data.
- I know the document says that transactions are globally unique. This program is defensive in that transactions that have an identifier may only execute once **per client**. Duplicate transaction IDs are rejected.


## Technical

- Dispute checks are O(1) lookup time.
- Checking for previously resolved disputes is O(1) lookup time.
- Finding referenced transactions for dispute, chargeback, and resolve transaction types are O(1) lookup time.
- I tried to be very careful about memory usage. That said, this program does keep some transaction information in memory, aggregating whenever possible.
- We could write a very low memory requirement version of this program. Instead of keeping transaction information in memory, we could go to disk. The low memory, disk-heavy approach could use an LRU cache to minimize performance impact.
- I am not an [architecture astronaut](https://www.joelonsoftware.com/2001/04/21/dont-let-architecture-astronauts-scare-you/), and hopefully, you'll see that I strive for [simplicity](https://grugbrain.dev/#grug-on-complexity).
- I've written a bit of Rust code on the job. I enjoy working with the language and want to do more work with Rust. If I need to be a pro on day one, I'm probably not your guy. I'll catch up quickly, though.
- Tests are provided in tests.rs. These are not unit tests. I know what unit tests are, and these tests provided me with the best bang for the buck.
