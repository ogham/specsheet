# Specsheet › xtests

This is Specsheet’s integration test suite. (The ‘x’ stands for ‘integration’.)

As tempting as it would be, Specsheet does not actually test itself. The integration test suite comes in the form of a Ruby script that runs Specsheet with various parameters and check documents.

It’s meant to be run inside a Vagrant VM, and a Vagrantfile has been provided for this purpose.
