#!/bin/sh
#
# This script should get run before the integration tests are run. It creates
# files and systemd services used within the test suite.


# The directory to place all the files in.
TEST_DIR='/testcases'

# Where files to copy are kept.
TEST_FILES='/vagrant/xtests/servers'

# We need to know the user that runs the tests.
DEVELOPER='vagrant'

# We also need an UID and a GID that are guaranteed to not exist, to
# test what happen when they don’t.
INVALID_UID=666
INVALID_GID=616


if ! test -d /vagrant; then
  echo "This is meant to be run on a Vagrant machine" > /dev/stderr
  exit 9
fi


echo "Getting rid of existing test cases..."

rm -rf $TEST_DIR
mkdir -p $TEST_DIR
chmod 777 $TEST_DIR



# Create the environment for the fs check tests.

echo "Creating file kind test cases..."

touch "$TEST_DIR/a-real-file-that-exists"
mkdir -p "$TEST_DIR/a-directory-this-time"

ln -s "$TEST_DIR/a-real-file-that-exists" "$TEST_DIR/a-symlink"
ln -s "$TEST_DIR/a-directory-this-time"   "$TEST_DIR/another-symlink"

mkdir "$TEST_DIR/specials"
sudo mknod "$TEST_DIR/specials/block-device" b  3 60
sudo mknod "$TEST_DIR/specials/char-device"  c 14 40
sudo mknod "$TEST_DIR/specials/named-pipe"   p

mkdir "$TEST_DIR/permissions"
for perms in 000 001 002 004 010 020 040 100 200 400 644 \
             755 777 1000 1001 2000 2010 4000 4100 7666 7777; do
  touch        "$TEST_DIR/permissions/$perms"
  chmod $perms "$TEST_DIR/permissions/$perms"
done

mkdir "$TEST_DIR/passwd"
touch                              "$TEST_DIR/passwd/unknown-uid"
touch                              "$TEST_DIR/passwd/unknown-gid"
sudo chown $INVALID_UID:$DEVELOPER "$TEST_DIR/passwd/unknown-uid"
sudo chown $DEVELOPER:$INVALID_GID "$TEST_DIR/passwd/unknown-gid"


# Create the environment for the systemd check tests.

echo "Spawning test servers..."

mkdir -p "$TEST_DIR/udp"
cp $TEST_FILES/udp-server.rb "$TEST_DIR/udp/"
cp $TEST_FILES/udp-server.service /etc/systemd/system/

mkdir -p "$TEST_DIR/http"
cp $TEST_FILES/http-server.rb "$TEST_DIR/http/"
cp $TEST_FILES/http-server.service /etc/systemd/system/

cp $TEST_FILES/returner.rb "$TEST_DIR/"
chmod +x "$TEST_DIR/returner.rb"

systemctl daemon-reload
systemctl enable udp-server
systemctl restart udp-server
systemctl enable http-server
systemctl restart http-server
