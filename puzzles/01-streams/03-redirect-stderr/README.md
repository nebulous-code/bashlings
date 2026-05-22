# Silencing errors while keeping output

There is a script at `/puzzle/noisy.sh` that prints useful output to stdout
and grumbles on stderr. Run it so that the useful output ends up in
`/puzzle/clean.txt` and **nothing** of the stderr noise leaks into that file
or onto your screen.

Hint: the noise on stderr is not an actual failure — you don't need to fix
`noisy.sh`. You just need to redirect the streams correctly.
