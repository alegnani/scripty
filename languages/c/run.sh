
echo "$(</dev/stdin)" > main.c 
gcc -o snippet main.c
./snippet
