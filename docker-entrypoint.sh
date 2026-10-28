#!/bin/sh
# Starts finstats as an unprivileged user that can write its data directory.
#
# When the bind-mounted data folder does not exist yet, Docker creates it owned by root, and an
# unprivileged process then cannot create the database in it. So the container starts as root for
# exactly one job: make the data directory belong to the user finstats will run as. Then it drops
# privileges for good and never runs finstats itself as root.
#
#   PUID / PGID   the user and group to run as (default 1000:1000); match the owner of your files
#   --user …      start the container as that user instead; then nothing is changed and nothing is dropped
set -e

# Everything FinStats writes (the database with the Jellyfin key, backups with everybody's history, uploads with
# other services' tokens) is for the user it runs as alone, whatever it is.
umask 077

DATA="${FINSTATS_DATA_DIR:-/data}"

if [ "$(id -u)" != 0 ]; then
    exec finstats "$@"
fi

PUID="${PUID:-1000}"
PGID="${PGID:-1000}"
case "$PUID$PGID" in *[!0-9]*) echo "finstats: PUID and PGID must be numbers (got '$PUID' and '$PGID')" >&2; exit 64 ;; esac

mkdir -p "$DATA"
# Only the folder itself and what FinStats keeps in it, never a recursive walk over everything there: the folder is
# the unprivileged user's to write, so root following it would hand that user whatever they had placed or linked into
# it. Inside FinStats' own folders a symbolic link is never followed (find does not, and chown -h changes the link
# itself) and a file with another name elsewhere (a hard link) is left alone. Only what someone else owns is touched:
# going over a large poster cache on every start would be slow, and pointless.
own() {
    find "$@" -xdev \( -type d -o -type l -o \( -type f -links 1 \) \) \( ! -user "$PUID" -o ! -group "$PGID" \) \
        -exec chown -h "$PUID:$PGID" {} + 2>/dev/null
}
(
    chown -h "$PUID:$PGID" "$DATA" && chmod go-rwx "$DATA" || exit 1
    for entry in finstats.db finstats.db-wal finstats.db-shm finstats.db-journal backups pre-update-backups geoip cache \
                 tautulli-upload.db restore-upload.tmp jellystat-upload.tmp streamystats-upload.tmp; do
        if [ -e "$DATA/$entry" ] || [ -L "$DATA/$entry" ]; then own "$DATA/$entry" || exit 1; fi
    done
) || echo "finstats: could not change the owner of $DATA (read-only or rootless mount?); continuing as $PUID:$PGID" >&2

exec su-exec "$PUID:$PGID" finstats "$@"
