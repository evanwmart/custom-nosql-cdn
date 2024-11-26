#!/bin/bash

# Exit immediately if a command exits with a non-zero status.

# Timestamp for the current benchmark run
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Create results directory
RESULTS_DIR="benchmark_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

# Clean up any previous benchmark data
rm -f bench_data.db data.db

# Build the project with profiling symbols
cargo build --profile release-with-debug

# Start the server in the background
./target/release-with-debug/custom-nosql-cdn &
SERVER_PID=$!

# Ensure the server is terminated when the script ends
trap "sudo kill $SERVER_PID $PERF_PID 2>/dev/null || true" EXIT

# Wait for the server to start
sleep 2

# Start perf recording on the server process
sudo perf record -F 99 -g --call-graph=dwarf -o "$RESULTS_DIR/perf.data" --pid $SERVER_PID &
PERF_PID=$!

# Number of total requests to send
NUM_REQUESTS=10000  # Adjusted to 10000 for testing; you can increase it as needed

# Number of concurrent processes
CONCURRENCY_LEVEL=10

# Requests per process
REQUESTS_PER_PROCESS=$((NUM_REQUESTS / CONCURRENCY_LEVEL))

# Function to monitor CPU and memory usage
monitor_usage() {
    echo "timestamp cpu mem vsz rss" > "$RESULTS_DIR/usage.dat"
    while kill -0 $SERVER_PID 2>/dev/null; do
        TIMESTAMP_SEC=$(date +%s)
        # Use LC_NUMERIC=C to ensure consistent decimal points
        USAGE=$(LC_NUMERIC=C ps -p $SERVER_PID -o %cpu,%mem,vsz,rss --no-headers)
        if [ -z "$USAGE" ]; then
            echo "Failed to get usage data at $TIMESTAMP_SEC" >> "$RESULTS_DIR/monitor.log"
        else
            echo "$USAGE" | awk -v ts=$TIMESTAMP_SEC '{print ts" "$1" "$2" "$3" "$4}' >> "$RESULTS_DIR/usage.dat"
        fi
        sleep 1
    done
}

# Start monitoring in the background
monitor_usage &
MONITOR_PID=$!

# Benchmark insert performance with parallel requests
echo "Benchmarking insert performance with concurrency level $CONCURRENCY_LEVEL..."
START_TIME=$(date +%s%N)

insert_worker() {
    local start_index=$1
    local end_index=$2
    for ((i=start_index; i<=end_index; i++)); do
        key="key$i"
        value="value$i"
        curl -s "http://127.0.0.1:8081/insert/$key/$value" > /dev/null
    done
}

# Launch insert workers in parallel
for ((p=0; p<CONCURRENCY_LEVEL; p++)); do
    start_index=$((p * REQUESTS_PER_PROCESS + 1))
    end_index=$(((p + 1) * REQUESTS_PER_PROCESS))
    insert_worker $start_index $end_index &
    PIDS[$p]=$!
    echo "Started insert worker $p for keys $start_index to $end_index"
done

# Wait for all insert workers to finish
for pid in ${PIDS[*]}; do
    wait $pid
done

END_TIME=$(date +%s%N)
DURATION_INSERT=$((($END_TIME - $START_TIME)/1000000))
echo "Total time for $NUM_REQUESTS inserts with concurrency $CONCURRENCY_LEVEL: ${DURATION_INSERT} ms"

# Save insert performance results
echo "Insert performance: Total time ${DURATION_INSERT} ms" > "$RESULTS_DIR/insert_performance.txt"

# Clear PIDS array
unset PIDS

# Benchmark get performance with parallel requests
echo "Benchmarking get performance with concurrency level $CONCURRENCY_LEVEL..."
START_TIME=$(date +%s%N)

get_worker() {
    local start_index=$1
    local end_index=$2
    for ((i=start_index; i<=end_index; i++)); do
        key="key$i"
        curl -s "http://127.0.0.1:8081/get/$key" > /dev/null
    done
}

# Launch get workers in parallel
for ((p=0; p<CONCURRENCY_LEVEL; p++)); do
    start_index=$((p * REQUESTS_PER_PROCESS + 1))
    end_index=$(((p + 1) * REQUESTS_PER_PROCESS))
    get_worker $start_index $end_index &
    PIDS[$p]=$!
    echo "Started get worker $p for keys $start_index to $end_index"
done

# Wait for all get workers to finish
for pid in ${PIDS[*]}; do
    wait $pid
done

END_TIME=$(date +%s%N)
DURATION_GET=$((($END_TIME - $START_TIME)/1000000))
echo "Total time for $NUM_REQUESTS gets with concurrency $CONCURRENCY_LEVEL: ${DURATION_GET} ms"

# Save get performance results
echo "Get performance: Total time ${DURATION_GET} ms" > "$RESULTS_DIR/get_performance.txt"

# Stop monitoring
kill $MONITOR_PID 2>/dev/null || true

# Stop perf recording gracefully
if ! sudo kill -INT $PERF_PID 2>/dev/null; then
    echo "Perf recording failed."
    exit 1
fi

wait $PERF_PID

# Check perf.data
if [ ! -s "$RESULTS_DIR/perf.data" ]; then
    echo "perf.data is empty or invalid. Skipping visualization."
    exit 1
fi

# Change so that we have permission to read file $RESULTS_DIR/perf.data
sudo chmod 644 "$RESULTS_DIR/perf.data"

# Generate visualization
hotspot "$RESULTS_DIR/perf.data" || echo "Hotspot visualization skipped."

echo "Finished."
