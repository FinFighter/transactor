# Transactor

**Transactor is a command line utility for processing account actions from CSV files.**

---

## CSV Input Format

```
type,   client,  tx,   amount
deposit,     1,   1,      1.5
deposit,     2,   2,      2.0
deposit,     1,   3,      2.0
withdrawal,  1,   4,      1.5
withdrawal,  2,   5,      3.0
```
### Aavailable Account Operations (type column):
- deposit
- withdrawal
- dispute
- resolve
- chargeback

---

## Usage

Provide the executable a path to a valid CSV file matching the specification above:

```
cargo run -- infile.csv > outfile.csv
```
---

## CSV Output Format

```
client,   available,  held,   total,  locked
     1,         1.5,   0.0,     1.5,   false
     2,         2.0,   0.0,     2.0,   false
```
---

## Assumptions

- Disputes only apply to previous deposit transactions.
   - Reasoning: A dispute can only be valid if the funds are able to be held. If a dispute were to be placed on a withdrawal  transaction, funds would need to be added to the account. This action is not supported.
   - A Dispute against a transaction that exceeds the account's available funds is ignored.
- Transaction IDs are unique, but not necessarily ordered.
- Relationship between a client ID and a transaction ID are consistent.
   - Example: If client 1 is associated with transaction 2, a dispute targeting transaction 2 can only be assigned to client 1.
- Accounts that are locked as a result of a chargeback operation cannot be updated. There is no mechanism to unlock an account.
- Numeric decimal values are assumed to be accurate to 4 decimal places. Additional digits after the ten thousandths place will not be considered and will not be rounded.

---

## Error Handling

Transactor tracks hard and soft errors. Hard errors will halt the execution of the program due to a critical failure. Soft errors do not halt the execution of the program, but rather silently refuse an account operation.

### Hard Errors
- Deserialization Errors
- IO Errors

### Soft Errors
- Attempting to withdraw more funds than available in the account.
- Attempting to dispute a previous deposit transaction that exceeds the current funds in the account.
- Attempting to interact with a previously locked account in any way.
- Attempting to withdrawal, dispute, resolve, or chargeback against a non-existent account.
- Attempting to dispute a non-existent transaction.
- Duplicating a transaction ID.
- Attempting to dispute an already disputed transaction.
- Attempting to resolve a non-disputed transaction.