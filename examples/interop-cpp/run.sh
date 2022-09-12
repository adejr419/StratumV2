#! /bin/sh

touch libsv2_ffi.a
touch a.out

# CLEAN
rm -f libsv2_ffi.a
rm -f a.out
rm -f sv2.h

cargo build --release -p sv2_ffi && cp ../../target/release/libsv2_ffi.a ./
../../build_header.sh

g++ -I ./ ./template-provider/template-provider.cpp  libsv2_ffi.a  -lpthread -ldl

./a.out &
provider_pid=$!

cargo run &
run_pid=$!

echo "run pid is $run_pid; provider pid is $provider_pid"

sleep 30

if ps -p $provider_pid > /dev/null && ps -p $run_pid > /dev/null
then
    echo "Success"
    kill $provider_pid
    kill $run_pid
    exit 1
else
    echo "Failure!!!"
    exit 1
fi
