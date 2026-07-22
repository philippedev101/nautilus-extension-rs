#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

required_tools=(
    dbus-run-session
    nautilus
    strings
    Xvfb
)

for tool in "${required_tools[@]}"; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for this validation" >&2
        exit 1
    fi
done

nautilus_bin="$(command -v nautilus)"
if ! strings "$nautilus_bin" | grep -F NAUTILUS_4_EXTENSION_DIR >/dev/null 2>&1; then
    wrapped_bin="$(strings "$nautilus_bin" | grep -m1 '/bin/\.nautilus-wrapped' || true)"
    if [ -z "$wrapped_bin" ] \
        || [ ! -x "$wrapped_bin" ] \
        || ! strings "$wrapped_bin" | grep -F NAUTILUS_4_EXTENSION_DIR >/dev/null 2>&1; then
        cat >&2 <<'EOF'
This Nautilus build does not advertise the NixOS NAUTILUS_4_EXTENSION_DIR
override. Refusing to run because the smoke test cannot guarantee an isolated
extension directory.
EOF
        exit 2
    fi
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/real-nautilus-sandbox-smoke}"
export NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4=1
unset NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG

bash scripts/validation/check-example-symbols.sh

work_dir="$(mktemp -d -t nautilus-extension-rs-gui-smoke.XXXXXX)"
cleanup() {
    if [ -n "${xvfb_pid:-}" ] && kill -0 "$xvfb_pid" >/dev/null 2>&1; then
        kill "$xvfb_pid" >/dev/null 2>&1 || true
        wait "$xvfb_pid" >/dev/null 2>&1 || true
    fi
    rm -rf "$work_dir"
}
trap cleanup EXIT

home_dir="$work_dir/home"
extension_dir="$work_dir/extensions-4"
files_dir="$work_dir/files"
runtime_dir="$work_dir/runtime"
tmp_dir="$work_dir/tmp"
log_file="$work_dir/nautilus.log"
xvfb_log="$work_dir/xvfb.log"

mkdir -p "$home_dir" "$extension_dir" "$files_dir" "$runtime_dir" "$tmp_dir"
chmod 700 "$runtime_dir"
printf 'nautilus-extension-rs GUI smoke\n' >"$files_dir/sample.txt"

for so in "$CARGO_TARGET_DIR"/debug/examples/lib*_provider.so; do
    cp "$so" "$extension_dir/"
done

display_num="${NAUTILUS_EXTENSION_RS_SANDBOX_DISPLAY:-128}"
while [ -e "/tmp/.X${display_num}-lock" ]; do
    display_num=$((display_num + 1))
done

Xvfb ":$display_num" -screen 0 1280x800x24 -nolisten tcp >"$xvfb_log" 2>&1 &
xvfb_pid=$!
sleep 1

if ! kill -0 "$xvfb_pid" >/dev/null 2>&1; then
    echo "Xvfb failed to start" >&2
    sed -n '1,160p' "$xvfb_log" >&2
    exit 1
fi

export DISPLAY=":$display_num"
export HOME="$home_dir"
export TMPDIR="$tmp_dir"
export XDG_CACHE_HOME="$work_dir/cache"
export XDG_CONFIG_HOME="$work_dir/config"
export XDG_DATA_HOME="$work_dir/data"
export XDG_RUNTIME_DIR="$runtime_dir"
export GSETTINGS_BACKEND=memory
export GTK_A11Y=none
export NO_AT_BRIDGE=1
export NAUTILUS_4_EXTENSION_DIR="$extension_dir"
export NAUTILUS_EXTENSION_RS_SMOKE_G_DEBUG="${NAUTILUS_EXTENSION_RS_SMOKE_G_DEBUG:-fatal-criticals}"
unset G_DEBUG

dbus_status=0
dbus-run-session -- bash -c '
set -euo pipefail

env \
    G_DEBUG="$NAUTILUS_EXTENSION_RS_SMOKE_G_DEBUG" \
    LD_DEBUG=libs,files \
    LD_DEBUG_OUTPUT="$3/ld-debug" \
    nautilus "$1" >"$2" 2>&1 &
nautilus_pid=$!

end_time=$((SECONDS + ${NAUTILUS_EXTENSION_RS_GUI_SETTLE_SECONDS:-6}))
while [ "$SECONDS" -lt "$end_time" ]; do
    if ! kill -0 "$nautilus_pid" >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

if kill -0 "$nautilus_pid" >/dev/null 2>&1; then
    kill "$nautilus_pid" >/dev/null 2>&1 || true
fi

status=0
wait "$nautilus_pid" || status=$?
if [ "$status" -ne 0 ] && [ "$status" -ne 143 ]; then
    exit "$status"
fi
' bash "$files_dir" "$log_file" "$work_dir" || dbus_status=$?

if [ "$dbus_status" -ne 0 ]; then
    echo "real-nautilus-sandbox-smoke: Nautilus failed with status $dbus_status" >&2
    echo "----- Nautilus log -----" >&2
    sed -n '1,200p' "$log_file" >&2 || true
    echo "----- Xvfb log -----" >&2
    sed -n '1,120p' "$xvfb_log" >&2 || true
    exit "$dbus_status"
fi

for example in column_provider menu_provider properties_model_provider; do
    if ! grep -R "lib${example}\\.so" "$work_dir"/ld-debug.* >/dev/null 2>&1; then
        echo "real-nautilus-sandbox-smoke: Nautilus did not load lib${example}.so" >&2
        echo "----- Nautilus log -----" >&2
        sed -n '1,200p' "$log_file" >&2 || true
        exit 1
    fi
done

echo "real-nautilus-sandbox-smoke: Nautilus launched with isolated display, bus, home, and extension dir"
