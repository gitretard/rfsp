# RFSP (Retarded file sending protocol) standard

# TODO! Implement actual file trasnfer
Current state: Metadata works.
Please note that ive never made a fucking protocol before.


Metadata packet (512 bytes):
The metadata packet will be sent to the receiver at the start of sending the file.
0-7 bytes: u64 size in bytes
8-512 bytes: Filename (will be truncated if longer than that) // Kept at 1 packet for the sake of simplicity


Data packet (512 bytes):
0-7 bytes: The Hash of the entire packet (u64 rust default hasher) 
8 byte: Opcode (1:Cancel,2:Finished: do final op) // More opcodes will be added
9-17 bytes: Packet count that starts as 0. so if the same packet gets sent again ignore the packet (u32)  (Thanks tom scott)
18-512 bytes: The actual file

For each packet the receiver will send the following back to the sender
1: Everything is okay. Continue as normal, 2: Resend (Err || Mismatched hash). 3: Cancelled by user (Q)

If the hash of the received packet isnt the same as the first 8 bytes then the receiver will will send 1 to sender and the sender will resend the packet again if it still isnt the same for the 3rd time the operation will be cancelled

If the opcode is 1. The receiver will close the connection and delete the downloaded file


If the opcode is 2 then
the sender will send the hash of the file and the receiver must check the hash of the file if the hash of the file is incorrect then resend it 1 more time 
if the file still doesnt have the correct hash. the operation is cancelled by the sender by saying whatever the sender likes (128 bytes limit) then closing the connection

Actual file sent in each packet = 494

All of these packets will be in a u8 array with the length of 512
