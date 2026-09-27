# shell.nix
#
# Classic (non-flake) dev shell: spins up a throwaway MariaDB instance for
# sqlx testing, provisioned via the same env-var contract the official
# `mariadb` container image uses (MARIADB_ROOT_PASSWORD / MARIADB_DATABASE /
# MARIADB_USER / MARIADB_PASSWORD).
#
# IMPORTANT: nixpkgs' `mariadb` package is just the upstream server binary.
# It does NOT read these env vars itself -- that provisioning logic lives in
# a separate project (docker-library/mariadb's docker-entrypoint.sh), which
# is what actually runs inside the official container image. This shellHook
# re-implements the same *interface* for local parity; it is not literally
# the code that will run in your production sidecar.
#
# Usage:
#   nix-shell
#   # or with direnv, in .envrc:
#   #   use nix
#
# Override provisioning before entering the shell if you want non-default
# creds (only matters on a FRESH .dev-mariadb/ -- see note below):
#   MARIADB_ROOT_PASSWORD=... MARIADB_DATABASE=... \
#   MARIADB_USER=...          MARIADB_PASSWORD=... nix-shell

{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = with pkgs; [
    mariadb
    sqlx-cli
    pkg-config
    openssl
  ]; # trim the rust-toolchain entries if you already manage those elsewhere

  shellHook = ''
    # ---- container-style provisioning inputs, with dev defaults ----
    if [ -z "$MARIADB_ROOT_PASSWORD" ]; then MARIADB_ROOT_PASSWORD=rootpass; fi
    if [ -z "$MARIADB_DATABASE" ]; then MARIADB_DATABASE=myapp; fi
    if [ -z "$MARIADB_USER" ]; then MARIADB_USER=myapp; fi
    if [ -z "$MARIADB_PASSWORD" ]; then MARIADB_PASSWORD=devpassword; fi
    export MARIADB_ROOT_PASSWORD MARIADB_DATABASE MARIADB_USER MARIADB_PASSWORD

    # ---- instance paths, local to this checkout ----
    export MYSQL_HOME="$PWD/.dev-mariadb"
    export MYSQL_DATADIR="$MYSQL_HOME/data"
    export MYSQL_SOCK="$MYSQL_HOME/mysqld.sock"
    export MYSQL_PID="$MYSQL_HOME/mysqld.pid"

    FIRST_RUN=0
    if [ ! -d "$MYSQL_DATADIR" ]; then
      FIRST_RUN=1
      mkdir -p "$MYSQL_DATADIR"
      mariadb-install-db --auth-root-authentication-method=normal \
        --datadir="$MYSQL_DATADIR" --basedir="${pkgs.mariadb}" >/dev/null
    fi

    mariadbd --datadir="$MYSQL_DATADIR" --socket="$MYSQL_SOCK" \
      --pid-file="$MYSQL_PID" --port=3306 --bind-address=127.0.0.1 &
    disown 2>/dev/null || true   # helps under direnv, which lacks job control

    # wait on the socket file rather than a logged-in query -- after the
    # first run, root requires a password, so we can't rely on a bare
    # `select 1` succeeding here.
    until [ -S "$MYSQL_SOCK" ]; do
      sleep 0.2
    done
    sleep 0.3   # small grace period after the socket appears

    # ---- one-time provisioning, only on a genuinely fresh datadir ----
    # (mirrors the container: env vars are honored once, against an empty
    # volume; a pre-existing datadir keeps whatever was set the first time)
    if [ "$FIRST_RUN" = "1" ]; then
      mariadb --socket="$MYSQL_SOCK" -u root <<SQL
ALTER USER 'root'@'localhost' IDENTIFIED BY '$MARIADB_ROOT_PASSWORD';
CREATE DATABASE IF NOT EXISTS \`$MARIADB_DATABASE\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER IF NOT EXISTS '$MARIADB_USER'@'%' IDENTIFIED BY '$MARIADB_PASSWORD';
GRANT ALL PRIVILEGES ON \`$MARIADB_DATABASE\`.* TO '$MARIADB_USER'@'%';
FLUSH PRIVILEGES;
SQL
      echo "Provisioned fresh MariaDB: db=$MARIADB_DATABASE user=$MARIADB_USER"
    fi

    export DATABASE_URL="mysql://$MARIADB_USER:$MARIADB_PASSWORD@127.0.0.1:3306/$MARIADB_DATABASE"
    echo "MariaDB ready -> $DATABASE_URL"

    # shut down by pid, not by authenticated admin command, so this still
    # works even if MARIADB_ROOT_PASSWORD doesn't match a pre-existing datadir
    trap 'kill "$(cat "$MYSQL_PID" 2>/dev/null)" 2>/dev/null' EXIT
  '';
}
