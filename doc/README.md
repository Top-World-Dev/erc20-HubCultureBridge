# System Logical Flow

## Registration Command Sequence
| CALLER   |   COMMAND  | HANDLER |   ACTION                            | RESULT                                                      |
|----------|------------|---------|-------------------------------------|-------------------------------------------------------------|
|HCWebsite |preregister |HCBackend|preregister called against Authority | Authority returns proof to HCBackend which is given to user |
|HCWebsite |register    |HCWeb3JS |transaction to be sent to blockchain | Registration event is issued then transaction is mined      |
|blockchain|registration|EventSys |Event handler to call HC backend     | Wallet may be assigned as owned to this user in HC backend  |

## Unregister Command Sequence
| CALLER   |   COMMAND  | HANDLER |   ACTION                            | RESULT                                                      |
|----------|------------|---------|-------------------------------------|-------------------------------------------------------------|
|HCWebsite |unregister  |HCBackend|preregister called against Authority | Authority submits unregister command to blockchain          |
|blockchain|unregister  |EventSys |Event handler to call HC backend     | Wallet may be unassigned from user in HC backend            |

## Deposit Command Sequence
| CALLER   |   COMMAND  | HANDLER |   ACTION                            | RESULT                                                      |
|----------|------------|---------|-------------------------------------|-------------------------------------------------------------|
|HCWebsite |deposit     |HCBackend|deposit called against Authority     | Authority submits deposit command to blockchain             |
|blockchain|Pending     |EventSys |Event handler to call HC backend     | Deposit may be marked as pending as scheduled for clearance |

## Clearance Command Sequence
| CALLER   |   COMMAND    | HANDLER |   ACTION                            | RESULT                                                      |
|----------|--------------|---------|-------------------------------------|-------------------------------------------------------------|
|HCBackend |releaseDeposit|HCBackend|releaseDeposit called against Vault  | Vault submits releaseDeposit command to blockchain          |
|blockchain|Deposit       |EventSys |Event handler to call HC backend     | Deposit may be marked as cleared                            |


## Decline Command Sequence
| CALLER   |   COMMAND    | HANDLER |   ACTION                            | RESULT                                                      |
|----------|--------------|---------|-------------------------------------|-------------------------------------------------------------|
|HCBackend |revokeDeposit |HCBackend|revokeDeposit called against Vault   | Vault submits revokeDeposit command to blockchain           |
|blockchain|Decline       |EventSys |Event handler to call HC backend     | Deposit may be marked as declined                           |

## Withdraw Command Sequence
| CALLER   |   COMMAND    | HANDLER |   ACTION                            | RESULT                                                     |
|----------|--------------|---------|-------------------------------------|-------------------------------------------------------------|
|HCWebsite |withdraw      |HCWeb3JS |transaction to be sent to blockchain | Withdraw event is issued then transaction is mined          |
|blockchain|Withdraw      |EventSys |Event handler to call HC backend     | Balance may be credited back to user in HCBackend           |
