# RFSP (Retarded file sending protocol) standard

Status: the code kinda works now but is extremely hacky and more dodgy than Marvin Gay Jr's father figure. see 
main.rs:143 
main.rs:168
main.rs:55
for example

# Will appreciate any help

# Issues: null bytes on EOF without sender's finish opcode (problem in file.read?)

# Please note that ive never made a fucking protocol before.

Usage: cargo run . -- mode

mode: send and recv

Metadata packet (512 bytes):
The metadata packet will be sent to the receiver at the start of sending the file.
0-8 bytes: u64 size in bytes
8-12 bytes: idempt
13-511 bytes: Filename (will be truncated if longer than that) // Kept at 1 packet for the sake of simplicity


Data packet (512 bytes):
0-8 bytes: The Hash of the entire packet (u64 rust default hasher) 
9 byte: Opcode (1: Cancel, 2:Finished: close conn)
10-14 bytes: Packet count that starts as 0. so if the same packet gets sent again ignore the packet (u32) (idempt) (Thanks tom scott)
15-512 bytes: The actual file

For each packet the receiver will send the following back to the sender
1: Everything is okay. Continue as normal. 2: Reserved 3: Cancel

If the hash of the received packet isnt the same as the first 8 bytes then the receiver will just cancel (Behavior might be changed soon)

If the opcode is 2. The receiver will close the connection


Actual file sent in each packet = 498

All of these packets will be in a u8 array with the length of 512
